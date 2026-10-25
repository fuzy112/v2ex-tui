use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::api::{Member, Notification, Reply, RssItem, Topic, V2exClient};
use crate::browser::Browser;
use crate::state::{AggregateState, NodeState, NotificationState, TokenState, TopicState, UiState};
use crate::ui::{
    render_empty, render_error, render_loading, render_status_bar, render_token_input,
};
use crate::views::aggregate::AggregateView;
use crate::views::help::HelpView;
use crate::views::node_select::NodeSelectView;
use crate::views::notifications::NotificationsView;
use crate::views::profile::ProfileView;
use crate::views::topic_detail::TopicDetailView;
use crate::views::topic_list::TopicListView;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    TopicList,
    TopicDetail,
    Notifications,
    Profile,
    Help,
    NodeSelect,
    TokenInput,
    Aggregate,
}

/// Result of a background request, applied on the UI thread.
#[derive(Debug)]
pub enum AppEvent {
    Topics {
        request_id: u64,
        page: i32,
        append: bool,
        focus_new: bool,
        result: Result<Vec<Topic>, String>,
    },
    Topic {
        request_id: u64,
        result: Result<Box<Topic>, String>,
    },
    Replies {
        request_id: u64,
        page: i32,
        append: bool,
        focus_new: bool,
        result: Result<Vec<Reply>, String>,
    },
    Notifications {
        request_id: u64,
        result: Result<Vec<Notification>, String>,
    },
    Profile {
        request_id: u64,
        result: Result<Box<Member>, String>,
    },
    Aggregate {
        request_id: u64,
        tab: String,
        result: Result<Vec<RssItem>, String>,
    },
}

#[derive(Debug)]
pub struct App {
    pub view: View,
    pub topic_state: TopicState,
    pub notification_state: NotificationState,
    pub profile: Option<Member>,
    pub node_state: NodeState,
    pub token_state: TokenState,
    pub ui_state: UiState,
    pub aggregate_state: AggregateState,
    pub terminal_width: usize,
    pub terminal_height: usize,
    // History navigation
    pub view_history: Vec<View>,
    pub history_position: usize,
    // Background loading
    event_tx: Option<mpsc::UnboundedSender<AppEvent>>,
    pending_loads: usize,
    topics_request_id: u64,
    detail_request_id: u64,
    replies_request_id: u64,
    notifications_request_id: u64,
    profile_request_id: u64,
    aggregate_request_id: u64,
    pending_open_next_topic: bool,
}

impl App {
    pub fn new() -> Self {
        let initial_view = View::Aggregate;
        Self {
            view: initial_view,
            topic_state: TopicState::default(),
            notification_state: NotificationState::default(),
            profile: None,
            node_state: NodeState::new(),
            token_state: TokenState::default(),
            ui_state: UiState::new(),
            aggregate_state: AggregateState::new(),
            terminal_width: 80,  // Default width
            terminal_height: 24, // Default height
            view_history: vec![initial_view],
            history_position: 0,
            event_tx: None,
            pending_loads: 0,
            topics_request_id: 0,
            detail_request_id: 0,
            replies_request_id: 0,
            notifications_request_id: 0,
            profile_request_id: 0,
            aggregate_request_id: 0,
            pending_open_next_topic: false,
        }
    }

    pub fn set_event_sender(&mut self, tx: mpsc::UnboundedSender<AppEvent>) {
        self.event_tx = Some(tx);
    }

    pub fn handle_resize(&mut self, width: usize) {
        self.terminal_width = width;
        if self.topic_state.current.is_some() {
            self.topic_state.detect_links(width);
        }
    }

    fn next_request_id(counter: &mut u64) -> u64 {
        *counter = counter.wrapping_add(1);
        *counter
    }

    fn begin_load(&mut self) {
        self.pending_loads += 1;
        self.ui_state.loading = true;
    }

    fn finish_load(&mut self) {
        self.pending_loads = self.pending_loads.saturating_sub(1);
        self.ui_state.loading = self.pending_loads > 0;
    }

    // Data loading methods
    pub fn load_topics(&mut self, client: &V2exClient, append: bool) {
        self.load_topics_with_focus(client, append, false);
    }

    pub fn load_topics_with_focus(&mut self, client: &V2exClient, append: bool, focus_new: bool) {
        self.ui_state.error = None;
        self.begin_load();

        let request_id = Self::next_request_id(&mut self.topics_request_id);
        let node = self.node_state.current_node.clone();
        let page = self.node_state.page;
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client
                .get_node_topics(&node, page)
                .await
                .map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Topics {
                    request_id,
                    page,
                    append,
                    focus_new,
                    result,
                });
            }
        });
    }

    pub fn load_topic_detail(&mut self, client: &V2exClient, topic_id: i64) {
        self.ui_state.error = None;
        self.begin_load();

        let request_id = Self::next_request_id(&mut self.detail_request_id);
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client
                .get_topic(topic_id)
                .await
                .map(Box::new)
                .map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Topic { request_id, result });
            }
        });
    }

    pub fn load_topic_replies(&mut self, client: &V2exClient, topic_id: i64, append: bool) {
        self.load_topic_replies_with_focus(client, topic_id, append, false);
    }

    pub fn load_topic_replies_with_focus(
        &mut self,
        client: &V2exClient,
        topic_id: i64,
        append: bool,
        focus_new: bool,
    ) {
        self.ui_state.error = None;
        self.begin_load();

        if !append {
            self.topic_state.replies_page = 1;
        }

        let request_id = Self::next_request_id(&mut self.replies_request_id);
        let page = self.topic_state.replies_page;
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client
                .get_topic_replies(topic_id, page)
                .await
                .map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Replies {
                    request_id,
                    page,
                    append,
                    focus_new,
                    result,
                });
            }
        });
    }

    pub fn load_notifications(&mut self, client: &V2exClient) {
        self.ui_state.error = None;
        self.begin_load();

        let request_id = Self::next_request_id(&mut self.notifications_request_id);
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client.get_notifications(1).await.map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Notifications { request_id, result });
            }
        });
    }

    pub fn load_profile(&mut self, client: &V2exClient) {
        self.ui_state.error = None;
        self.begin_load();

        let request_id = Self::next_request_id(&mut self.profile_request_id);
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client
                .get_member()
                .await
                .map(Box::new)
                .map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Profile { request_id, result });
            }
        });
    }

    pub fn load_aggregate(&mut self, client: &V2exClient) {
        self.ui_state.error = None;
        self.begin_load();

        let request_id = Self::next_request_id(&mut self.aggregate_request_id);
        let tab = self.aggregate_state.current_tab.clone();
        let client = client.clone();
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = client.get_rss_feed(&tab).await.map_err(|e| e.to_string());
            if let Some(tx) = tx {
                let _ = tx.send(AppEvent::Aggregate {
                    request_id,
                    tab,
                    result,
                });
            }
        });
    }

    pub fn switch_aggregate_tab(&mut self, client: &V2exClient, tab: &str) {
        self.aggregate_state.switch_tab(tab);
        self.load_aggregate(client);
    }

    /// Apply a background request result. Always called on the UI thread.
    pub fn handle_event(&mut self, event: AppEvent, client: &V2exClient) {
        match event {
            AppEvent::Topics {
                request_id,
                page,
                append,
                focus_new,
                result,
            } => {
                self.finish_load();
                if request_id != self.topics_request_id {
                    return;
                }

                match result {
                    Ok(mut new_topics) => {
                        if append {
                            let prev_len = self.topic_state.topics.len();
                            if new_topics.is_empty() {
                                if self.node_state.page == page {
                                    self.node_state.page = (page - 1).max(1);
                                }
                                self.pending_open_next_topic = false;
                                self.ui_state.status_message =
                                    "Already at the last topic".to_string();
                                return;
                            }

                            let loaded = new_topics.len();
                            self.topic_state.topics.append(&mut new_topics);
                            if focus_new {
                                self.topic_state.selected = prev_len;
                            }
                            self.ui_state.status_message = format!(
                                "Loaded {} more topics (total: {}) from {}",
                                loaded,
                                self.topic_state.topics.len(),
                                self.node_state.current_node
                            );

                            if self.pending_open_next_topic {
                                self.pending_open_next_topic = false;
                                if let Some(topic) = self.topic_state.topics.get(prev_len) {
                                    let topic_id = topic.id;
                                    self.topic_state.current = None;
                                    self.topic_state.replies.clear();
                                    self.topic_state.reset_scroll();
                                    self.load_topic_detail(client, topic_id);
                                    self.load_topic_replies(client, topic_id, false);
                                }
                            }
                        } else {
                            self.topic_state.topics = new_topics;
                            self.topic_state.selected = 0;
                            self.ui_state.status_message = format!(
                                "Loaded {} topics from {}",
                                self.topic_state.topics.len(),
                                self.node_state.current_node
                            );
                        }
                    }
                    Err(e) => {
                        if append && self.node_state.page == page {
                            self.node_state.page = (page - 1).max(1);
                        }
                        self.pending_open_next_topic = false;
                        self.ui_state.error = Some(format!("Failed to load topics: {}", e));
                    }
                }
            }
            AppEvent::Topic { request_id, result } => {
                self.finish_load();
                if request_id != self.detail_request_id {
                    return;
                }

                match result {
                    Ok(topic) => {
                        let topic_id = topic.id;
                        self.topic_state.current = Some(*topic);
                        self.topic_state.detect_links(self.terminal_width);
                        self.ui_state.status_message = format!("Loaded topic {}", topic_id);
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to load topic: {}", e));
                    }
                }
            }
            AppEvent::Replies {
                request_id,
                page,
                append,
                focus_new,
                result,
            } => {
                self.finish_load();
                if request_id != self.replies_request_id {
                    return;
                }

                match result {
                    Ok(replies) => {
                        let replies_len = replies.len();
                        if append {
                            if replies.is_empty() {
                                self.ui_state.status_message =
                                    "Already at the last reply".to_string();
                                return;
                            }

                            let prev_len = self.topic_state.replies.len();
                            self.topic_state.replies.extend(replies);
                            if focus_new {
                                self.topic_state.selected_reply = prev_len;
                                self.topic_state.replies_list_state.select(Some(prev_len));
                            }
                            self.ui_state.status_message = format!(
                                "Loaded {} more replies (total: {})",
                                replies_len,
                                self.topic_state.replies.len()
                            );
                        } else {
                            self.topic_state.replies = replies;
                            self.topic_state.selected_reply = 0;
                            if self.topic_state.replies.is_empty() {
                                self.topic_state.replies_list_state.select(None);
                            } else {
                                self.topic_state.replies_list_state.select(Some(0));
                            }
                            self.ui_state.status_message =
                                format!("Loaded {} replies", replies_len);
                        }

                        self.topic_state.replies_page = page + 1;
                        self.topic_state.detect_links(self.terminal_width);
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to load replies: {}", e));
                    }
                }
            }
            AppEvent::Notifications { request_id, result } => {
                self.finish_load();
                if request_id != self.notifications_request_id {
                    return;
                }

                match result {
                    Ok(notifications) => {
                        self.notification_state.notifications = notifications;
                        self.notification_state.selected = 0;
                        self.ui_state.status_message = format!(
                            "Loaded {} notifications",
                            self.notification_state.notifications.len()
                        );
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to load notifications: {}", e));
                    }
                }
            }
            AppEvent::Profile { request_id, result } => {
                self.finish_load();
                if request_id != self.profile_request_id {
                    return;
                }

                match result {
                    Ok(member) => {
                        self.profile = Some(*member);
                        self.ui_state.status_message = "Loaded profile".to_string();
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to load profile: {}", e));
                    }
                }
            }
            AppEvent::Aggregate {
                request_id,
                tab,
                result,
            } => {
                self.finish_load();
                if request_id != self.aggregate_request_id {
                    return;
                }

                match result {
                    Ok(items) => {
                        self.aggregate_state.items = items;
                        self.aggregate_state.selected = 0;
                        self.ui_state.status_message = format!(
                            "Loaded {} aggregated topics from {} tab",
                            self.aggregate_state.items.len(),
                            tab
                        );
                    }
                    Err(e) => {
                        self.ui_state.error =
                            Some(format!("Failed to load aggregated topics: {}", e));
                    }
                }
            }
        }
    }

    // Helper to find current topic index based on previous view
    fn find_current_topic_index_in_previous_view(&self) -> Option<(usize, View)> {
        let current_topic_id = self.topic_state.current.as_ref()?.id;

        // Check history stack to determine the source view
        let source_view = if self.history_position > 0 {
            self.view_history.get(self.history_position - 1).copied()
        } else {
            None
        };

        match source_view {
            Some(View::Aggregate) => {
                // Find in aggregate items
                self.aggregate_state
                    .items
                    .iter()
                    .position(|item| item.extract_topic_id() == Some(current_topic_id))
                    .map(|index| (index, View::Aggregate))
            }
            Some(View::TopicList) => {
                // Find in topic list
                self.topic_state
                    .find_current_topic_index()
                    .map(|index| (index, View::TopicList))
            }
            Some(View::Notifications) => {
                // Find in notifications (if applicable)
                self.topic_state
                    .find_current_topic_index()
                    .map(|index| (index, View::TopicList))
            }
            _ => None,
        }
    }

    // Topic navigation in detail view
    /// Move to the next topic, loading the next page when at the end of the list.
    pub fn next_topic_or_load(&mut self, client: &V2exClient) {
        let at_last = self.topic_state.selected + 1 >= self.topic_state.topics.len();
        if at_last && !self.topic_state.topics.is_empty() {
            if self.ui_state.loading {
                return;
            }
            self.node_state.page += 1;
            self.load_topics_with_focus(client, true, true);
        } else {
            self.topic_state.next_topic();
        }
    }

    /// Move to the next reply, loading the next page when at the end.
    pub fn next_reply_or_load(&mut self, client: &V2exClient) {
        if self.topic_state.show_replies && !self.topic_state.replies.is_empty() {
            let at_last = self.topic_state.selected_reply + 1 >= self.topic_state.replies.len();
            let has_more = self.topic_state.replies.len()
                < self
                    .topic_state
                    .current
                    .as_ref()
                    .map(|t| t.replies as usize)
                    .unwrap_or(0);

            if at_last && has_more {
                if self.ui_state.loading {
                    return;
                }
                if let Some(topic_id) = self.topic_state.current.as_ref().map(|t| t.id) {
                    self.load_topic_replies_with_focus(client, topic_id, true, true);
                }
            } else if at_last {
                self.ui_state.status_message = "Already at the last reply".to_string();
            } else {
                self.topic_state.next_reply(self.terminal_width);
            }
        } else {
            self.topic_state.scroll_down();
        }
    }

    pub fn switch_to_next_topic(&mut self, client: &V2exClient) {
        if let Some((current_index, source_view)) = self.find_current_topic_index_in_previous_view()
        {
            match source_view {
                View::Aggregate => {
                    let items_len = self.aggregate_state.items.len();
                    if items_len == 0 {
                        return;
                    }
                    if current_index + 1 >= items_len {
                        // Already at last item
                        self.ui_state.status_message =
                            "Already at the last aggregated topic".to_string();
                        return;
                    }
                    let next_index = current_index + 1;
                    if let Some(next_item) = self.aggregate_state.items.get(next_index) {
                        if let Some(topic_id) = next_item.extract_topic_id() {
                            self.topic_state.current = None;
                            self.topic_state.replies.clear();
                            self.topic_state.reset_scroll();
                            self.load_topic_detail(client, topic_id);
                            self.load_topic_replies(client, topic_id, false);
                            self.ui_state.status_message =
                                format!("Switched to next aggregated topic (#{})", next_index + 1);
                        }
                    }
                }
                View::TopicList | View::Notifications => {
                    let topics_len = self.topic_state.topics.len();
                    if topics_len == 0 {
                        return;
                    }
                    if current_index + 1 >= topics_len {
                        // Try to load the next page, then open its first topic.
                        if self.ui_state.loading {
                            return;
                        }
                        self.node_state.page += 1;
                        self.pending_open_next_topic = true;
                        self.load_topics_with_focus(client, true, true);
                        return;
                    }
                    let next_index = current_index + 1;
                    if let Some(next_topic) = self.topic_state.topics.get(next_index) {
                        let topic_id = next_topic.id;
                        self.topic_state.current = None;
                        self.topic_state.replies.clear();
                        self.topic_state.reset_scroll();
                        self.load_topic_detail(client, topic_id);
                        self.load_topic_replies(client, topic_id, false);
                        self.ui_state.status_message =
                            format!("Switched to next topic (#{})", next_index + 1);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn switch_to_previous_topic(&mut self, client: &V2exClient) {
        if let Some((current_index, source_view)) = self.find_current_topic_index_in_previous_view()
        {
            match source_view {
                View::Aggregate => {
                    let items_len = self.aggregate_state.items.len();
                    if items_len == 0 {
                        return;
                    }
                    if current_index == 0 {
                        // Already at first item
                        self.ui_state.status_message =
                            "Already at the first aggregated topic".to_string();
                        return;
                    }
                    let prev_index = current_index - 1;
                    if let Some(prev_item) = self.aggregate_state.items.get(prev_index) {
                        if let Some(topic_id) = prev_item.extract_topic_id() {
                            self.topic_state.current = None;
                            self.topic_state.replies.clear();
                            self.topic_state.reset_scroll();
                            self.load_topic_detail(client, topic_id);
                            self.load_topic_replies(client, topic_id, false);
                            self.ui_state.status_message = format!(
                                "Switched to previous aggregated topic (#{})",
                                prev_index + 1
                            );
                        }
                    }
                }
                View::TopicList | View::Notifications => {
                    let topics_len = self.topic_state.topics.len();
                    if topics_len == 0 {
                        return;
                    }
                    if current_index == 0 {
                        // Already at first topic
                        self.ui_state.status_message = "Already at the first topic".to_string();
                        return;
                    }
                    let prev_index = current_index - 1;
                    if let Some(prev_topic) = self.topic_state.topics.get(prev_index) {
                        let topic_id = prev_topic.id;
                        self.topic_state.current = None;
                        self.topic_state.replies.clear();
                        self.topic_state.reset_scroll();
                        self.load_topic_detail(client, topic_id);
                        self.load_topic_replies(client, topic_id, false);
                        self.ui_state.status_message =
                            format!("Switched to previous topic (#{})", prev_index + 1);
                    }
                }
                _ => {}
            }
        }
    }

    // Browser methods
    pub fn open_current_topic_in_browser(&mut self) {
        if let Some(ref topic) = self.topic_state.current {
            match Browser::open_topic(topic.id) {
                Ok(result) => {
                    self.ui_state.status_message = result.to_string();
                }
                Err(e) => {
                    self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                }
            }
        }
    }

    pub fn open_selected_reply_in_browser(&mut self) {
        if let Some(ref topic) = self.topic_state.current {
            if let Some(reply) = self
                .topic_state
                .replies
                .get(self.topic_state.selected_reply)
            {
                match Browser::open_topic_reply(topic.id, reply.id) {
                    Ok(result) => {
                        self.ui_state.status_message = result.to_string();
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                    }
                }
            }
        }
    }

    pub fn copy_selected_reply_to_clipboard(&mut self) {
        if let Some(reply) = self
            .topic_state
            .replies
            .get(self.topic_state.selected_reply)
        {
            let content = reply
                .content_rendered
                .as_ref()
                .or(reply.content.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_default();

            // Strip HTML tags for plain text
            let plain_text = crate::util::html_to_text(&content, 80);

            match crate::clipboard::copy_to_clipboard(&plain_text) {
                Ok(()) => {
                    self.ui_state.status_message =
                        format!("Copied reply #{} to clipboard", reply.id);
                }
                Err(e) => {
                    self.ui_state.error = Some(format!("Failed to copy to clipboard: {}", e));
                }
            }
        } else {
            self.ui_state.status_message = "No reply selected".to_string();
        }
    }

    pub fn copy_topic_content_to_clipboard(&mut self, topic: &crate::api::Topic) {
        let content = topic
            .content_rendered
            .as_ref()
            .or(topic.content.as_ref())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Strip HTML tags for plain text
        let plain_text = crate::util::html_to_text(&content, 80);

        match crate::clipboard::copy_to_clipboard(&plain_text) {
            Ok(()) => {
                self.ui_state.status_message =
                    format!("Copied topic '{}' to clipboard", topic.title);
            }
            Err(e) => {
                self.ui_state.error = Some(format!("Failed to copy to clipboard: {}", e));
            }
        }
    }

    pub fn open_selected_topic_in_browser(&mut self) {
        if let Some(topic) = self.topic_state.topics.get(self.topic_state.selected) {
            match Browser::open_topic(topic.id) {
                Ok(result) => {
                    self.ui_state.status_message = result.to_string();
                }
                Err(e) => {
                    self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                }
            }
        }
    }

    pub fn open_notification_in_browser(&mut self) {
        if let Some(notification) = self
            .notification_state
            .notifications
            .get(self.notification_state.selected)
        {
            if let Some(topic_id) = notification.extract_topic_id() {
                match Browser::open_topic(topic_id) {
                    Ok(result) => {
                        self.ui_state.status_message = result.to_string();
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                    }
                }
            } else {
                self.ui_state.status_message =
                    "No topic link found in this notification".to_string();
            }
        }
    }

    pub fn open_selected_aggregate_in_browser(&mut self) {
        if let Some(item) = self
            .aggregate_state
            .items
            .get(self.aggregate_state.selected)
        {
            match Browser::open_url(&item.link) {
                Ok(result) => {
                    self.ui_state.status_message = result.to_string();
                }
                Err(e) => {
                    self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                }
            }
        }
    }

    #[allow(dead_code)] // Not currently used, but kept for future use
    pub fn open_detected_link(&mut self, shortcut: usize) {
        if let Some(link) = self.topic_state.get_link_by_shortcut(shortcut) {
            match Browser::open_url(link) {
                Ok(result) => {
                    self.ui_state.status_message = format!("Opening link {}: {}", shortcut, result);
                }
                Err(e) => {
                    self.ui_state.error = Some(format!("Failed to open link {}: {}", shortcut, e));
                }
            }
        } else {
            self.ui_state.status_message = format!("No link found for shortcut {}", shortcut);
        }
    }

    fn get_status_with_links(&self) -> String {
        if self.view == View::TopicDetail && !self.topic_state.link_shortcuts.is_empty() {
            let links_info = self.topic_state.link_shortcuts.join(", ");
            if self.ui_state.status_message.is_empty() {
                format!("Links: {}", links_info)
            } else {
                format!("{} | Links: {}", self.ui_state.status_message, links_info)
            }
        } else {
            self.ui_state.status_message.clone()
        }
    }

    // History navigation methods
    const MAX_HISTORY_SIZE: usize = 50;

    /// Navigate to a new view, mutating the history stack
    pub fn navigate_to(&mut self, view: View) {
        // Truncate forward history
        if self.history_position + 1 < self.view_history.len() {
            self.view_history.truncate(self.history_position + 1);
        }

        // Push new view
        self.view_history.push(view);
        self.history_position = self.view_history.len() - 1;

        // Enforce max history size
        if self.view_history.len() > Self::MAX_HISTORY_SIZE {
            let excess = self.view_history.len() - Self::MAX_HISTORY_SIZE;
            self.view_history.drain(0..excess);
            self.history_position -= excess;
        }

        if view == View::Help {
            self.ui_state.help_scroll = 0;
        }

        self.view = view;
    }

    /// Navigate backward in history (l key)
    pub fn history_back(&mut self) -> bool {
        if self.history_position > 0 {
            self.history_position -= 1;
            self.view = self.view_history[self.history_position];
            self.ui_state.error = None;
            true
        } else {
            false
        }
    }

    /// Navigate forward in history (r key)
    pub fn history_forward(&mut self) -> bool {
        if self.history_position + 1 < self.view_history.len() {
            self.history_position += 1;
            self.view = self.view_history[self.history_position];
            self.ui_state.error = None;
            true
        } else {
            false
        }
    }

    /// Remove current view from history and return the view to navigate to (q/Esc)
    /// Returns None if history is empty or no previous view exists (should exit app)
    pub fn remove_current_from_history(&mut self) -> Option<View> {
        if self.view_history.is_empty() {
            return None;
        }

        // If at the first position, there's no previous view to go back to
        if self.history_position == 0 {
            return None;
        }

        // Remove current view
        self.view_history.remove(self.history_position);

        if self.view_history.is_empty() {
            return None;
        }

        // Go to previous item in history (left)
        self.history_position -= 1;

        self.view = self.view_history[self.history_position];
        self.ui_state.error = None;
        Some(self.view)
    }

    // Rendering
    pub fn render(&mut self, frame: &mut Frame) {
        // Update terminal dimensions
        let terminal_size = frame.area();
        self.terminal_width = terminal_size.width as usize;
        self.terminal_height = terminal_size.height as usize;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.area());

        match self.view {
            View::TopicList => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
                } else if self.topic_state.topics.is_empty() {
                    render_empty(
                        frame,
                        chunks[0],
                        "Topics",
                        "No topics here. Press g to refresh or s to switch node.",
                        &self.ui_state.theme,
                    );
                } else {
                    let topic_list_view = TopicListView::new();
                    topic_list_view.render(
                        frame,
                        chunks[0],
                        &self.topic_state.topics,
                        self.topic_state.selected,
                        &self.node_state.current_node,
                        &self.ui_state.theme,
                    );
                }
            }
            View::TopicDetail => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
                } else if let Some(ref topic) = self.topic_state.current {
                    let topic_detail_view = TopicDetailView::new();
                    if self.topic_state.show_replies {
                        let area = chunks[0];
                        let is_narrow = area.width < 100;
                        let split_chunks = Layout::default()
                            .direction(if is_narrow {
                                Direction::Vertical
                            } else {
                                Direction::Horizontal
                            })
                            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                            .split(area);
                        topic_detail_view.render_split(
                            frame,
                            split_chunks[0],
                            split_chunks[1],
                            topic,
                            self.topic_state.scroll,
                            &self.topic_state.detected_links,
                            self.topic_state.link_input_state.is_active,
                            self.topic_state.parsed_content_cache.as_deref(),
                            &self.topic_state.replies,
                            &mut self.topic_state.replies_list_state,
                            &self.ui_state.theme,
                        );
                    } else {
                        topic_detail_view.render(
                            frame,
                            chunks[0],
                            topic,
                            self.topic_state.scroll,
                            &self.topic_state.detected_links,
                            self.topic_state.link_input_state.is_active,
                            self.topic_state.parsed_content_cache.as_deref(),
                            &self.ui_state.theme,
                        );
                    }
                } else {
                    render_empty(
                        frame,
                        chunks[0],
                        "Topic",
                        "No topic loaded. Press g to retry or q to go back.",
                        &self.ui_state.theme,
                    );
                }
            }
            View::Notifications => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
                } else if self.notification_state.notifications.is_empty() {
                    render_empty(
                        frame,
                        chunks[0],
                        "Notifications",
                        "No notifications. Press g to refresh.",
                        &self.ui_state.theme,
                    );
                } else {
                    let notifications_view = NotificationsView::new();
                    notifications_view.render(
                        frame,
                        chunks[0],
                        &self.notification_state.notifications,
                        self.notification_state.selected,
                        &self.ui_state.theme,
                    );
                }
            }
            View::Profile => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
                } else if let Some(ref profile) = self.profile {
                    let profile_view = ProfileView::new();
                    profile_view.render(frame, chunks[0], profile, &self.ui_state.theme);
                } else {
                    render_empty(
                        frame,
                        chunks[0],
                        "Profile",
                        "No profile loaded. Press g to refresh.",
                        &self.ui_state.theme,
                    );
                }
            }
            View::Help => {
                let help_view = HelpView::new();
                help_view.render(
                    frame,
                    chunks[0],
                    self.ui_state.help_scroll,
                    &self.ui_state.theme,
                );
            }
            View::NodeSelect => {
                if self.node_state.favorite_nodes.is_empty() {
                    render_empty(
                        frame,
                        chunks[0],
                        "Nodes",
                        "No matching nodes. Press Tab to switch input mode.",
                        &self.ui_state.theme,
                    );
                } else {
                    let node_select_view = NodeSelectView::new();
                    node_select_view.render(
                        frame,
                        chunks[0],
                        &self.node_state.favorite_nodes,
                        self.node_state.selected,
                        &self.node_state.current_node,
                        &self.node_state.completion_input,
                        self.node_state.completion_cursor,
                        self.node_state.is_completion_mode,
                        &self.ui_state.theme,
                    );
                }
            }
            View::TokenInput => {
                render_token_input(
                    frame,
                    chunks[0],
                    &self.token_state.input,
                    self.token_state.cursor,
                    &self.ui_state.theme,
                );
            }
            View::Aggregate => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
                } else if self.aggregate_state.items.is_empty() {
                    render_empty(
                        frame,
                        chunks[0],
                        "Aggregated Topics",
                        "No aggregated topics. Press g to refresh.",
                        &self.ui_state.theme,
                    );
                } else {
                    let aggregate_view = AggregateView::new();
                    aggregate_view.render(
                        frame,
                        chunks[0],
                        &self.aggregate_state.items,
                        self.aggregate_state.selected,
                        &self.aggregate_state.current_tab,
                        &self.ui_state.theme,
                    );
                }
            }
        }

        let status_message = self.get_status_with_links();
        render_status_bar(frame, chunks[1], &status_message, &self.ui_state.theme);
    }
}
