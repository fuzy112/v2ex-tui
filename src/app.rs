use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
};

use crate::api::{Member, Notification, Reply, Topic, V2exClient};
use crate::nodes::get_all_nodes;
use crate::ui::{
    render_error, render_help, render_loading, render_node_select, render_notifications,
    render_profile, render_replies, render_status_bar, render_token_input, render_topic_detail,
    render_topic_list, Theme,
};

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
    // Topic data
    pub topics: Vec<Topic>,
    pub selected_topic: usize,
    pub current_topic: Option<Topic>,
    pub topic_replies: Vec<Reply>,
    // Notification data
    pub notifications: Vec<Notification>,
    pub selected_notification: usize,
    // Profile data
    pub profile: Option<Member>,
    // Navigation state
    pub current_node: String,
    pub page: i32,
    pub replies_page: i32,
    // UI state
    pub loading: bool,
    pub error: Option<String>,
    pub status_message: String,
    pub show_replies: bool,
    pub theme: Theme,
    // Scroll positions
    pub topic_scroll: usize,
    pub selected_reply: usize,
    // Node selection
    pub favorite_nodes: Vec<(String, String)>,
    pub all_nodes: Vec<(String, String)>,
    pub original_favorite_nodes: Vec<(String, String)>,
    pub selected_node: usize,
    pub replies_list_state: ListState,
    // Token input
    pub token_input: String,
    pub token_cursor: usize,
    // Node completion input
    pub node_completion_input: String,
    pub node_completion_cursor: usize,
    pub is_node_completion_mode: bool,
}

impl App {
    pub fn new() -> Self {
        let favorite_nodes = vec![
            ("python".to_string(), "Python".to_string()),
            ("programmer".to_string(), "程序员".to_string()),
            ("share".to_string(), "分享发现".to_string()),
            ("create".to_string(), "分享创造".to_string()),
            ("jobs".to_string(), "酷工作".to_string()),
            ("go".to_string(), "Go 编程语言".to_string()),
            ("rust".to_string(), "Rust 编程语言".to_string()),
            ("javascript".to_string(), "JavaScript".to_string()),
            ("linux".to_string(), "Linux".to_string()),
        ];

        let all_nodes = get_all_nodes().to_vec();

        Self {
            view: View::TopicList,
            topics: Vec::new(),
            selected_topic: 0,
            current_topic: None,
            topic_replies: Vec::new(),
            notifications: Vec::new(),
            selected_notification: 0,
            profile: None,
            current_node: "python".to_string(),
            page: 1,
            replies_page: 1,
            loading: false,
            error: None,
            status_message: "Press '?' for help".to_string(),
            show_replies: false,
            theme: Theme::default(),
            topic_scroll: 0,
            selected_reply: 0,
            favorite_nodes: favorite_nodes.clone(),
            all_nodes,
            original_favorite_nodes: favorite_nodes,
            selected_node: 0,
            replies_list_state: ListState::default(),
            token_input: String::new(),
            token_cursor: 0,
            node_completion_input: String::new(),
            node_completion_cursor: 0,
            is_node_completion_mode: false,
        }
    }

    // Data loading methods
    pub async fn load_topics(&mut self, client: &V2exClient, append: bool) {
        self.loading = true;
        self.error = None;

        match client.get_node_topics(&self.current_node, self.page).await {
            Ok(mut new_topics) => {
                if append && self.page > 1 {
                    self.topics.append(&mut new_topics);
                    self.status_message = format!(
                        "Loaded {} more topics (total: {}) from {}",
                        new_topics.len(),
                        self.topics.len(),
                        self.current_node
                    );
                } else {
                    self.topics = new_topics;
                    self.selected_topic = 0;
                    self.status_message = format!(
                        "Loaded {} topics from {}",
                        self.topics.len(),
                        self.current_node
                    );
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to load topics: {}", e));
            }
        }

        self.loading = false;
    }

    pub async fn load_topic_detail(&mut self, client: &V2exClient, topic_id: i64) {
        self.loading = true;
        self.error = None;

        match client.get_topic(topic_id).await {
            Ok(topic) => {
                self.current_topic = Some(topic);
                self.status_message = format!("Loaded topic {}", topic_id);
            }
            Err(e) => {
                self.error = Some(format!("Failed to load topic: {}", e));
            }
        }

        self.loading = false;
    }

    pub async fn load_topic_replies(&mut self, client: &V2exClient, topic_id: i64, append: bool) {
        self.loading = true;
        self.error = None;

        if !append {
            self.replies_page = 1;
        }

        match client.get_topic_replies(topic_id, self.replies_page).await {
            Ok(replies) => {
                let replies_len = replies.len();
                let is_empty = replies.is_empty();
                if append && self.replies_page > 1 {
                    self.topic_replies.extend(replies);
                    self.status_message = format!(
                        "Loaded {} more replies (total: {})",
                        replies_len,
                        self.topic_replies.len()
                    );
                } else {
                    self.topic_replies = replies;
                    self.selected_reply = 0;
                    if is_empty {
                        self.replies_list_state.select(None);
                    } else {
                        self.replies_list_state.select(Some(0));
                    }
                    self.status_message = format!("Loaded {} replies", self.topic_replies.len());
                }

                if !is_empty {
                    self.replies_page += 1;
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to load replies: {}", e));
            }
        }

        self.loading = false;
    }

    pub async fn load_notifications(&mut self, client: &V2exClient) {
        self.loading = true;
        self.error = None;

        match client.get_notifications(1).await {
            Ok(notifications) => {
                self.notifications = notifications;
                self.selected_notification = 0;
                self.status_message = format!("Loaded {} notifications", self.notifications.len());
            }
            Err(e) => {
                self.error = Some(format!("Failed to load notifications: {}", e));
            }
        }

        self.loading = false;
    }

    pub async fn load_profile(&mut self, client: &V2exClient) {
        self.loading = true;
        self.error = None;

        match client.get_member().await {
            Ok(member) => {
                self.profile = Some(member);
                self.status_message = "Loaded profile".to_string();
            }
            Err(e) => {
                self.error = Some(format!("Failed to load profile: {}", e));
            }
        }

        self.loading = false;
    }

    // Navigation methods
    pub fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = (self.selected_topic + 1) % self.topics.len();
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = if self.selected_topic == 0 {
                self.topics.len() - 1
            } else {
                self.selected_topic - 1
            };
        }
    }

    pub fn next_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification =
                (self.selected_notification + 1) % self.notifications.len();
        }
    }

    pub fn previous_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification = if self.selected_notification == 0 {
                self.notifications.len() - 1
            } else {
                self.selected_notification - 1
            };
        }
    }

    pub fn switch_node(&mut self, node: &str) {
        self.current_node = node.to_string();
        self.page = 1;
    }

    // Scroll methods
    pub fn scroll_topic_up(&mut self) {
        if self.topic_scroll >= 3 {
            self.topic_scroll -= 3;
        } else {
            self.topic_scroll = 0;
        }
    }

    pub fn scroll_topic_down(&mut self) {
        self.topic_scroll += 3;
    }

    pub fn next_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = (self.selected_reply + 1) % self.topic_replies.len();
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    pub fn previous_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = if self.selected_reply == 0 {
                self.topic_replies.len() - 1
            } else {
                self.selected_reply - 1
            };
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    pub fn reset_scroll(&mut self) {
        self.topic_scroll = 0;
        self.selected_reply = 0;
        if self.topic_replies.is_empty() {
            self.replies_list_state.select(None);
        } else {
            self.replies_list_state.select(Some(0));
        }
    }

    // Node selection methods
    pub fn next_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected_node = (self.selected_node + 1) % self.favorite_nodes.len();
        }
    }

    pub fn previous_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected_node = if self.selected_node == 0 {
                self.favorite_nodes.len() - 1
            } else {
                self.selected_node - 1
            };
        }
    }

    pub fn select_current_node(&mut self) {
        if let Some((node_name, _)) = self.favorite_nodes.get(self.selected_node) {
            self.current_node = node_name.clone();
            self.page = 1;
        } else if self.is_node_completion_mode {
            let node_name = self.node_completion_input.trim();
            if !node_name.is_empty() {
                self.current_node = node_name.to_string();
                self.page = 1;
            }
        }
    }

    // Topic navigation in detail view
    fn find_current_topic_index(&self) -> Option<usize> {
        if let Some(current_topic) = &self.current_topic {
            self.topics
                .iter()
                .position(|topic| topic.id == current_topic.id)
        } else {
            None
        }
    }

    pub async fn switch_to_next_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.find_current_topic_index() {
            let next_index = (current_index + 1) % self.topics.len();
            if let Some(next_topic) = self.topics.get(next_index) {
                let topic_id = next_topic.id;
                self.current_topic = None;
                self.topic_replies.clear();
                self.reset_scroll();
                self.load_topic_detail(client, topic_id).await;
                self.load_topic_replies(client, topic_id, false).await;
                self.status_message = format!("Switched to next topic (#{})", next_index + 1);
            }
        }
    }

    pub async fn switch_to_previous_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.find_current_topic_index() {
            let prev_index = if current_index == 0 {
                self.topics.len() - 1
            } else {
                current_index - 1
            };
            if let Some(prev_topic) = self.topics.get(prev_index) {
                let topic_id = prev_topic.id;
                self.current_topic = None;
                self.topic_replies.clear();
                self.reset_scroll();
                self.load_topic_detail(client, topic_id).await;
                self.load_topic_replies(client, topic_id, false).await;
                self.status_message = format!("Switched to previous topic (#{})", prev_index + 1);
            }
        }
    }

    // Token input methods
    pub fn insert_token_char(&mut self, ch: char) {
        let byte_pos = self
            .token_input
            .char_indices()
            .nth(self.token_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.token_input.len());
        self.token_input.insert(byte_pos, ch);
        self.token_cursor += 1;
    }

    pub fn delete_token_char(&mut self) {
        if self.token_cursor > 0 {
            let byte_pos = self
                .token_input
                .char_indices()
                .nth(self.token_cursor - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next_byte_pos = self
                .token_input
                .char_indices()
                .nth(self.token_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.token_input.len());
            self.token_input.drain(byte_pos..next_byte_pos);
            self.token_cursor -= 1;
        }
    }

    pub fn move_token_cursor_left(&mut self) {
        if self.token_cursor > 0 {
            self.token_cursor -= 1;
        }
    }

    pub fn move_token_cursor_right(&mut self) {
        if self.token_cursor < self.token_input.chars().count() {
            self.token_cursor += 1;
        }
    }

    pub fn save_token(&self) -> Result<()> {
        let config_dir = V2exClient::config_dir()?;
        let token_path = config_dir.join("token.txt");
        std::fs::write(&token_path, self.token_input.trim())
            .with_context(|| format!("Failed to write token to {:?}", token_path))?;
        Ok(())
    }

    // Node input methods
    pub fn insert_node_char(&mut self, ch: char) {
        let byte_pos = self
            .node_completion_input
            .char_indices()
            .nth(self.node_completion_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.node_completion_input.len());
        self.node_completion_input.insert(byte_pos, ch);
        self.node_completion_cursor += 1;
        if self.is_node_completion_mode {
            self.update_node_suggestions();
        }
    }

    pub fn delete_node_char(&mut self) {
        if self.node_completion_cursor > 0 {
            let byte_pos = self
                .node_completion_input
                .char_indices()
                .nth(self.node_completion_cursor - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next_byte_pos = self
                .node_completion_input
                .char_indices()
                .nth(self.node_completion_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.node_completion_input.len());
            self.node_completion_input.drain(byte_pos..next_byte_pos);
            self.node_completion_cursor -= 1;
            if self.is_node_completion_mode {
                self.update_node_suggestions();
            }
        }
    }

    pub fn move_node_cursor_left(&mut self) {
        if self.node_completion_cursor > 0 {
            self.node_completion_cursor -= 1;
        }
    }

    pub fn move_node_cursor_right(&mut self) {
        if self.node_completion_cursor < self.node_completion_input.chars().count() {
            self.node_completion_cursor += 1;
        }
    }

    pub fn toggle_node_completion_mode(&mut self) {
        self.is_node_completion_mode = !self.is_node_completion_mode;
        if self.is_node_completion_mode {
            self.update_node_suggestions();
        } else {
            self.favorite_nodes = self.original_favorite_nodes.clone();
            self.selected_node = 0;
        }
    }

    fn update_node_suggestions(&mut self) {
        let input = self.node_completion_input.trim();
        if input.is_empty() {
            self.favorite_nodes = self.all_nodes.iter().take(20).cloned().collect();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored_nodes: Vec<((String, String), i64)> = self
                .all_nodes
                .iter()
                .filter_map(|(name, title)| {
                    let name_score = matcher.fuzzy_match(name, input);
                    let title_score = matcher.fuzzy_match(title, input);
                    let score = name_score.unwrap_or(0).max(title_score.unwrap_or(0));

                    if score > 0 {
                        Some(((name.clone(), title.clone()), score))
                    } else {
                        None
                    }
                })
                .collect();

            scored_nodes.sort_by(|a, b| b.1.cmp(&a.1));

            self.favorite_nodes = scored_nodes
                .into_iter()
                .take(20)
                .map(|(node, _)| node)
                .collect();
        }
        self.selected_node = 0;
    }

    pub fn reset_node_selection(&mut self) {
        self.node_completion_input.clear();
        self.node_completion_cursor = 0;
        self.is_node_completion_mode = false;
    }

    pub fn enter_completing_read_mode(&mut self) {
        self.view = View::NodeSelect;
        self.node_completion_input.clear();
        self.node_completion_cursor = 0;
        self.is_node_completion_mode = true;
        self.update_node_suggestions();
    }

    // Browser methods
    pub fn open_current_topic_in_browser(&mut self) {
        if let Some(ref topic) = self.current_topic {
            let url = format!("https://www.v2ex.com/t/{}", topic.id);
            match webbrowser::open(&url) {
                Ok(_) => {
                    self.status_message = format!("Opened topic {} in browser", topic.id);
                }
                Err(e) => {
                    self.error = Some(format!("Failed to open browser: {}", e));
                }
            }
        }
    }

    pub fn open_selected_reply_in_browser(&mut self) {
        if let Some(ref topic) = self.current_topic {
            if let Some(reply) = self.topic_replies.get(self.selected_reply) {
                let url = format!("https://www.v2ex.com/t/{}#r_{}", topic.id, reply.id);
                match webbrowser::open(&url) {
                    Ok(_) => {
                        self.status_message =
                            format!("Opened topic {} (reply #{}) in browser", topic.id, reply.id);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to open browser: {}", e));
                    }
                }
            }
        }
    }

    pub fn open_selected_topic_in_browser(&mut self) {
        if let Some(topic) = self.topics.get(self.selected_topic) {
            let url = format!("https://www.v2ex.com/t/{}", topic.id);
            match webbrowser::open(&url) {
                Ok(_) => {
                    self.status_message = format!("Opened topic {} in browser", topic.id);
                }
                Err(e) => {
                    self.error = Some(format!("Failed to open browser: {}", e));
                }
            }
        }
    }

    pub fn open_notification_in_browser(&mut self) {
        if let Some(notification) = self.notifications.get(self.selected_notification) {
            if let Some(topic_id) = notification.extract_topic_id() {
                let url = format!("https://www.v2ex.com/t/{}", topic_id);
                match webbrowser::open(&url) {
                    Ok(_) => {
                        self.status_message = format!("Opened topic {} in browser", topic_id);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to open browser: {}", e));
                    }
                }
            } else {
                self.status_message = "No topic link found in this notification".to_string();
            }
        }
    }

    // Rendering
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        match self.view {
            View::TopicList => {
                if self.loading {
                    render_loading(frame, chunks[0], &self.theme);
                } else if let Some(ref error) = self.error {
                    render_error(frame, chunks[0], error, &self.theme);
                } else {
                    render_topic_list(
                        frame,
                        chunks[0],
                        &self.topics,
                        self.selected_topic,
                        &self.current_node,
                        &self.theme,
                    );
                }
            }
            View::TopicDetail => {
                if self.loading {
                    render_loading(frame, chunks[0], &self.theme);
                } else if let Some(ref error) = self.error {
                    render_error(frame, chunks[0], error, &self.theme);
                } else if let Some(ref topic) = self.current_topic {
                    if self.show_replies {
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
                        render_topic_detail(
                            frame,
                            split_chunks[0],
                            topic,
                            self.topic_scroll,
                            &self.theme,
                        );
                        render_replies(
                            frame,
                            split_chunks[1],
                            &self.topic_replies,
                            &mut self.replies_list_state,
                            &self.theme,
                        );
                    } else {
                        render_topic_detail(
                            frame,
                            chunks[0],
                            topic,
                            self.topic_scroll,
                            &self.theme,
                        );
                    }
                }
            }
            View::Notifications => {
                if self.loading {
                    render_loading(frame, chunks[0], &self.theme);
                } else if let Some(ref error) = self.error {
                    render_error(frame, chunks[0], error, &self.theme);
                } else {
                    render_notifications(
                        frame,
                        chunks[0],
                        &self.notifications,
                        self.selected_notification,
                        &self.theme,
                    );
                }
            }
            View::Profile => {
                if self.loading {
                    render_loading(frame, chunks[0], &self.theme);
                } else if let Some(ref error) = self.error {
                    render_error(frame, chunks[0], error, &self.theme);
                } else if let Some(ref profile) = self.profile {
                    render_profile(frame, chunks[0], profile, &self.theme);
                }
            }
            View::Help => {
                render_help(frame, chunks[0], &self.theme);
            }
            View::NodeSelect => {
                render_node_select(
                    frame,
                    chunks[0],
                    &self.favorite_nodes,
                    self.selected_node,
                    &self.current_node,
                    &self.node_completion_input,
                    self.node_completion_cursor,
                    self.is_node_completion_mode,
                    &self.theme,
                );
            }
            View::TokenInput => {
                render_token_input(
                    frame,
                    chunks[0],
                    &self.token_input,
                    self.token_cursor,
                    &self.theme,
                );
            }
        }

        render_status_bar(frame, chunks[1], &self.status_message, &self.theme);
    }
}
