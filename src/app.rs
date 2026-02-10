use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::api::{Member, V2exClient};
use crate::browser::Browser;
use crate::state::{AggregateState, NodeState, NotificationState, TokenState, TopicState, UiState};
use crate::ui::{render_error, render_loading, render_status_bar, render_token_input};
use crate::views::aggregate::AggregateView;
use crate::views::help::HelpView;
use crate::views::node_select::NodeSelectView;
use crate::views::notifications::NotificationsView;
use crate::views::profile::ProfileView;
use crate::views::topic_detail::TopicDetailView;
use crate::views::topic_list::TopicListView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        }
    }

    // Data loading methods
    pub async fn load_topics(&mut self, client: &V2exClient, append: bool) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        match client
            .get_node_topics(&self.node_state.current_node, self.node_state.page)
            .await
        {
            Ok(mut new_topics) => {
                if append && self.node_state.page > 1 {
                    self.topic_state.topics.append(&mut new_topics);
                    self.ui_state.status_message = format!(
                        "Loaded {} more topics (total: {}) from {}",
                        new_topics.len(),
                        self.topic_state.topics.len(),
                        self.node_state.current_node
                    );
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
                self.ui_state.error = Some(format!("Failed to load topics: {}", e));
            }
        }

        self.ui_state.loading = false;
    }

    pub async fn load_topic_detail(&mut self, client: &V2exClient, topic_id: i64) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        match client.get_topic(topic_id).await {
            Ok(topic) => {
                self.topic_state.current = Some(topic);
                self.topic_state.detect_links(self.terminal_width);
                self.ui_state.status_message = format!("Loaded topic {}", topic_id);
            }
            Err(e) => {
                self.ui_state.error = Some(format!("Failed to load topic: {}", e));
            }
        }

        self.ui_state.loading = false;
    }

    pub async fn load_topic_replies(&mut self, client: &V2exClient, topic_id: i64, append: bool) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        if !append {
            self.topic_state.replies_page = 1;
        }

        match client
            .get_topic_replies(topic_id, self.topic_state.replies_page)
            .await
        {
            Ok(replies) => {
                let replies_len = replies.len();
                let is_empty = replies.is_empty();
                if append && self.topic_state.replies_page > 1 {
                    self.topic_state.replies.extend(replies);
                    self.ui_state.status_message = format!(
                        "Loaded {} more replies (total: {})",
                        replies_len,
                        self.topic_state.replies.len()
                    );
                } else {
                    self.topic_state.replies = replies;
                    self.topic_state.selected_reply = 0;
                    if is_empty {
                        self.topic_state.replies_list_state.select(None);
                    } else {
                        self.topic_state.replies_list_state.select(Some(0));
                    }
                    self.ui_state.status_message = format!("Loaded {} replies", replies_len);
                }
                self.topic_state.replies_page += 1;
                // Update links after loading replies
                self.topic_state.detect_links(self.terminal_width);
            }
            Err(e) => {
                self.ui_state.error = Some(format!("Failed to load replies: {}", e));
            }
        }

        self.ui_state.loading = false;
    }

    pub async fn load_notifications(&mut self, client: &V2exClient) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        match client.get_notifications(1).await {
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

        self.ui_state.loading = false;
    }

    pub async fn load_profile(&mut self, client: &V2exClient) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        match client.get_member().await {
            Ok(member) => {
                self.profile = Some(member);
                self.ui_state.status_message = "Loaded profile".to_string();
            }
            Err(e) => {
                self.ui_state.error = Some(format!("Failed to load profile: {}", e));
            }
        }

        self.ui_state.loading = false;
    }

    pub async fn load_aggregate(&mut self, client: &V2exClient) {
        self.ui_state.loading = true;
        self.ui_state.error = None;

        match client.get_rss_feed(&self.aggregate_state.current_tab).await {
            Ok(items) => {
                self.aggregate_state.items = items;
                self.aggregate_state.selected = 0;
                self.ui_state.status_message = format!(
                    "Loaded {} aggregated topics from {} tab",
                    self.aggregate_state.items.len(),
                    self.aggregate_state.current_tab
                );
            }
            Err(e) => {
                self.ui_state.error = Some(format!("Failed to load aggregated topics: {}", e));
            }
        }

        self.ui_state.loading = false;
    }

    pub async fn switch_aggregate_tab(&mut self, client: &V2exClient, tab: &str) {
        self.aggregate_state.switch_tab(tab);
        self.load_aggregate(client).await;
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
    pub async fn switch_to_next_topic(&mut self, client: &V2exClient) {
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
                            self.load_topic_detail(client, topic_id).await;
                            self.load_topic_replies(client, topic_id, false).await;
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
                    let at_last = current_index + 1 >= topics_len;
                    let next_index = if at_last {
                        // Try to load more topics
                        let prev_page = self.node_state.page;
                        self.node_state.page += 1;
                        let prev_len = self.topic_state.topics.len();
                        self.load_topics(client, true).await;
                        if self.topic_state.topics.len() > prev_len {
                            // New topics loaded, move to next
                            prev_len
                        } else {
                            // No more topics to load, stay at current position and restore page
                            self.node_state.page = prev_page;
                            self.ui_state.error = None; // Clear the API error
                            self.ui_state.status_message = "Already at the last topic".to_string();
                            return;
                        }
                    } else {
                        current_index + 1
                    };
                    if let Some(next_topic) = self.topic_state.topics.get(next_index) {
                        let topic_id = next_topic.id;
                        self.topic_state.current = None;
                        self.topic_state.replies.clear();
                        self.topic_state.reset_scroll();
                        self.load_topic_detail(client, topic_id).await;
                        self.load_topic_replies(client, topic_id, false).await;
                        self.ui_state.status_message =
                            format!("Switched to next topic (#{})", next_index + 1);
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn switch_to_previous_topic(&mut self, client: &V2exClient) {
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
                            self.load_topic_detail(client, topic_id).await;
                            self.load_topic_replies(client, topic_id, false).await;
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
                        self.load_topic_detail(client, topic_id).await;
                        self.load_topic_replies(client, topic_id, false).await;
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
            let plain_text = html2text::from_read(content.as_bytes(), 80);

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
        let plain_text = html2text::from_read(content.as_bytes(), 80);

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

        self.view = view;
        self.ui_state.error = None;
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
                }
            }
            View::Notifications => {
                if self.ui_state.loading {
                    render_loading(frame, chunks[0], &self.ui_state.theme);
                } else if let Some(ref error) = self.ui_state.error {
                    render_error(frame, chunks[0], error, &self.ui_state.theme);
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
                }
            }
            View::Help => {
                let help_view = HelpView::new();
                help_view.render(frame, chunks[0], &self.ui_state.theme);
            }
            View::NodeSelect => {
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
