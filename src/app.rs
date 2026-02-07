use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::api::{Member, V2exClient};
use crate::state::{NodeState, NotificationState, TokenState, TopicState, UiState};
use crate::ui::{
    render_error, render_help, render_loading, render_node_select, render_profile,
    render_status_bar, render_token_input,
};
use crate::views::notifications::NotificationsView;
use crate::views::profile::ProfileView;
use crate::views::topic_detail::TopicDetailView;
use crate::views::topic_list::TopicListView;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    TopicList,
    TopicDetail,
    Notifications,
    Profile,
    Help,
    NodeSelect,
    TokenInput,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::TopicList,
            topic_state: TopicState::default(),
            notification_state: NotificationState::default(),
            profile: None,
            node_state: NodeState::new(),
            token_state: TokenState::default(),
            ui_state: UiState::new(),
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
                    self.ui_state.status_message =
                        format!("Loaded {} replies", self.topic_state.replies.len());
                }

                if !is_empty {
                    self.topic_state.replies_page += 1;
                }
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

    // Topic navigation in detail view
    pub async fn switch_to_next_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.topic_state.find_current_topic_index() {
            let next_index = (current_index + 1) % self.topic_state.topics.len();
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
    }

    pub async fn switch_to_previous_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.topic_state.find_current_topic_index() {
            let prev_index = if current_index == 0 {
                self.topic_state.topics.len() - 1
            } else {
                current_index - 1
            };
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
    }

    // Browser methods
    pub fn open_current_topic_in_browser(&mut self) {
        if let Some(ref topic) = self.topic_state.current {
            let url = format!("https://www.v2ex.com/t/{}", topic.id);
            match webbrowser::open(&url) {
                Ok(_) => {
                    self.ui_state.status_message = format!("Opened topic {} in browser", topic.id);
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
                let url = format!("https://www.v2ex.com/t/{}#r_{}", topic.id, reply.id);
                match webbrowser::open(&url) {
                    Ok(_) => {
                        self.ui_state.status_message =
                            format!("Opened topic {} (reply #{}) in browser", topic.id, reply.id);
                    }
                    Err(e) => {
                        self.ui_state.error = Some(format!("Failed to open browser: {}", e));
                    }
                }
            }
        }
    }

    pub fn open_selected_topic_in_browser(&mut self) {
        if let Some(topic) = self.topic_state.topics.get(self.topic_state.selected) {
            let url = format!("https://www.v2ex.com/t/{}", topic.id);
            match webbrowser::open(&url) {
                Ok(_) => {
                    self.ui_state.status_message = format!("Opened topic {} in browser", topic.id);
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
                let url = format!("https://www.v2ex.com/t/{}", topic_id);
                match webbrowser::open(&url) {
                    Ok(_) => {
                        self.ui_state.status_message =
                            format!("Opened topic {} in browser", topic_id);
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

    // Rendering
    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

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
                render_help(frame, chunks[0], &self.ui_state.theme);
            }
            View::NodeSelect => {
                render_node_select(
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
        }

        render_status_bar(
            frame,
            chunks[1],
            &self.ui_state.status_message,
            &self.ui_state.theme,
        );
    }
}
