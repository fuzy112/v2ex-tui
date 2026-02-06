use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
    Frame, Terminal,
};
use std::io;

mod api;
mod nodes;
mod ui;

use api::{Member, Notification, Reply, Topic, V2exClient};
use nodes::get_all_nodes;
use ui::{
    render_error, render_help, render_loading, render_node_select, render_notifications,
    render_profile, render_replies, render_reply_input, render_status_bar, render_token_input,
    render_topic_detail, render_topic_list, Theme,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    TopicList,
    TopicDetail,
    Notifications,
    Profile,
    Help,
    NodeSelect,
    TokenInput,
    ReplyInput,
}

#[derive(Debug)]
struct App {
    view: View,
    topics: Vec<Topic>,
    selected_topic: usize,
    current_topic: Option<Topic>,
    topic_replies: Vec<Reply>,
    notifications: Vec<Notification>,
    selected_notification: usize,
    profile: Option<Member>,
    current_node: String,
    page: i32,
    replies_page: i32,
    loading: bool,
    error: Option<String>,
    status_message: String,
    show_replies: bool,
    theme: Theme,
    // Scroll positions
    topic_scroll: usize,
    selected_reply: usize,
    // Node selection
    favorite_nodes: Vec<(String, String)>, // Current nodes to display (favorites or suggestions)
    all_nodes: Vec<(String, String)>,      // All available nodes for autocompletion
    original_favorite_nodes: Vec<(String, String)>, // Original favorite nodes (9 nodes)
    selected_node: usize,
    // List state for replies
    replies_list_state: ListState,
    // Token input
    token_input: String,
    token_cursor: usize,
    // Node selection manual input
    node_completion_input: String,
    node_completion_cursor: usize,
    is_node_completion_mode: bool,
    // Reply input
    reply_input: String,
    reply_cursor: usize,
}

impl App {
    fn new() -> Self {
        // Define favorite nodes (9 nodes for quick access)
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

        // All available nodes for autocompletion (1333 nodes)
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
            reply_input: String::new(),
            reply_cursor: 0,
        }
    }

    async fn load_topics(&mut self, client: &V2exClient, append: bool) {
        self.loading = true;
        self.error = None;

        match client.get_node_topics(&self.current_node, self.page).await {
            Ok(mut new_topics) => {
                if append && self.page > 1 {
                    // Append new topics to existing ones
                    self.topics.append(&mut new_topics);
                    self.status_message = format!(
                        "Loaded {} more topics (total: {}) from {}",
                        new_topics.len(),
                        self.topics.len(),
                        self.current_node
                    );
                } else {
                    // Replace topics (first page or not appending)
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

    async fn load_topic_detail(&mut self, client: &V2exClient, topic_id: i64) {
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

    async fn load_topic_replies(&mut self, client: &V2exClient, topic_id: i64, append: bool) {
        self.loading = true;
        self.error = None;

        // Reset page if not appending
        if !append {
            self.replies_page = 1;
        }

        match client.get_topic_replies(topic_id, self.replies_page).await {
            Ok(replies) => {
                let replies_len = replies.len();
                let is_empty = replies.is_empty();
                if append && self.replies_page > 1 {
                    // Append new replies to existing ones
                    self.topic_replies.extend(replies);
                    self.status_message = format!(
                        "Loaded {} more replies (total: {})",
                        replies_len,
                        self.topic_replies.len()
                    );
                } else {
                    // Replace replies (first page or not appending)
                    self.topic_replies = replies;
                    self.selected_reply = 0;
                    if is_empty {
                        self.replies_list_state.select(None);
                    } else {
                        self.replies_list_state.select(Some(0));
                    }
                    self.status_message = format!("Loaded {} replies", self.topic_replies.len());
                }

                // Increment page for next load if we got replies
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

    async fn load_notifications(&mut self, client: &V2exClient) {
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

    async fn load_profile(&mut self, client: &V2exClient) {
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

    fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = (self.selected_topic + 1) % self.topics.len();
        }
    }

    fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = if self.selected_topic == 0 {
                self.topics.len() - 1
            } else {
                self.selected_topic - 1
            };
        }
    }

    fn next_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification =
                (self.selected_notification + 1) % self.notifications.len();
        }
    }

    fn previous_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification = if self.selected_notification == 0 {
                self.notifications.len() - 1
            } else {
                self.selected_notification - 1
            };
        }
    }

    fn switch_node(&mut self, node: &str) {
        self.current_node = node.to_string();
        self.page = 1;
    }

    // Scroll methods
    fn scroll_topic_up(&mut self) {
        if self.topic_scroll >= 3 {
            self.topic_scroll -= 3;
        } else {
            self.topic_scroll = 0;
        }
    }

    fn scroll_topic_down(&mut self) {
        self.topic_scroll += 3; // Scroll 3 lines at a time for better performance
    }

    fn next_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = (self.selected_reply + 1) % self.topic_replies.len();
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    fn previous_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = if self.selected_reply == 0 {
                self.topic_replies.len() - 1
            } else {
                self.selected_reply - 1
            };
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    fn reset_scroll(&mut self) {
        self.topic_scroll = 0;
        self.selected_reply = 0;
        if self.topic_replies.is_empty() {
            self.replies_list_state.select(None);
        } else {
            self.replies_list_state.select(Some(0));
        }
    }

    // Node selection methods
    fn next_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected_node = (self.selected_node + 1) % self.favorite_nodes.len();
        }
    }

    fn previous_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected_node = if self.selected_node == 0 {
                self.favorite_nodes.len() - 1
            } else {
                self.selected_node - 1
            };
        }
    }

    fn select_current_node(&mut self) {
        // First try to use the selected candidate from the list
        if let Some((node_name, _)) = self.favorite_nodes.get(self.selected_node) {
            self.current_node = node_name.clone();
            self.page = 1;
        } else if self.is_node_completion_mode {
            // In completion mode, fallback to manual input if no selection available
            let node_name = self.node_completion_input.trim();
            if !node_name.is_empty() {
                self.current_node = node_name.to_string();
                self.page = 1;
            }
        }
    }

    // Find current topic index in the topics list
    fn find_current_topic_index(&self) -> Option<usize> {
        if let Some(current_topic) = &self.current_topic {
            self.topics
                .iter()
                .position(|topic| topic.id == current_topic.id)
        } else {
            None
        }
    }

    // Switch to next topic in detail view
    async fn switch_to_next_topic(&mut self, client: &V2exClient) {
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

    // Switch to previous topic in detail view
    async fn switch_to_previous_topic(&mut self, client: &V2exClient) {
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
    fn insert_token_char(&mut self, ch: char) {
        let byte_pos = self
            .token_input
            .char_indices()
            .nth(self.token_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.token_input.len());
        self.token_input.insert(byte_pos, ch);
        self.token_cursor += 1;
    }

    fn delete_token_char(&mut self) {
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

    fn move_token_cursor_left(&mut self) {
        if self.token_cursor > 0 {
            self.token_cursor -= 1;
        }
    }

    fn move_token_cursor_right(&mut self) {
        if self.token_cursor < self.token_input.chars().count() {
            self.token_cursor += 1;
        }
    }

    fn save_token(&self) -> Result<()> {
        let config_dir = V2exClient::config_dir()?;
        let token_path = config_dir.join("token.txt");
        std::fs::write(&token_path, self.token_input.trim())
            .with_context(|| format!("Failed to write token to {:?}", token_path))?;
        Ok(())
    }

    // Node manual input methods
    fn insert_node_char(&mut self, ch: char) {
        let byte_pos = self
            .node_completion_input
            .char_indices()
            .nth(self.node_completion_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.node_completion_input.len());
        self.node_completion_input.insert(byte_pos, ch);
        self.node_completion_cursor += 1;
        // Update suggestions after inserting character
        if self.is_node_completion_mode {
            self.update_node_suggestions();
        }
    }

    fn delete_node_char(&mut self) {
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
            // Update suggestions after deleting character
            if self.is_node_completion_mode {
                self.update_node_suggestions();
            }
        }
    }

    fn move_node_cursor_left(&mut self) {
        if self.node_completion_cursor > 0 {
            self.node_completion_cursor -= 1;
        }
    }

    fn move_node_cursor_right(&mut self) {
        if self.node_completion_cursor < self.node_completion_input.chars().count() {
            self.node_completion_cursor += 1;
        }
    }

    fn toggle_node_completion_mode(&mut self) {
        self.is_node_completion_mode = !self.is_node_completion_mode;
        if self.is_node_completion_mode {
            // Entering manual mode, update suggestions based on current input
            self.update_node_suggestions();
        } else {
            // Exiting manual mode, restore original favorite nodes
            self.favorite_nodes = self.original_favorite_nodes.clone();
            self.selected_node = 0;
        }
    }

    fn update_node_suggestions(&mut self) {
        let input = self.node_completion_input.trim();
        if input.is_empty() {
            // If input is empty, show all nodes (or maybe top N nodes)
            // For now, show first 20 nodes from all_nodes
            self.favorite_nodes = self.all_nodes.iter().take(20).cloned().collect();
        } else {
            // Use Skim's fuzzy matching algorithm (V2)
            let matcher = SkimMatcherV2::default();
            let mut scored_nodes: Vec<((String, String), i64)> = self
                .all_nodes
                .iter()
                .filter_map(|(name, title)| {
                    // Try matching against both name and title
                    let name_score = matcher.fuzzy_match(name, input);
                    let title_score = matcher.fuzzy_match(title, input);

                    // Take the higher score
                    let score = name_score.unwrap_or(0).max(title_score.unwrap_or(0));

                    if score > 0 {
                        Some(((name.clone(), title.clone()), score))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score descending (higher score = better match)
            scored_nodes.sort_by(|a, b| b.1.cmp(&a.1));

            // Take top 20 matches
            self.favorite_nodes = scored_nodes
                .into_iter()
                .take(20)
                .map(|(node, _)| node)
                .collect();
        }
        self.selected_node = 0;
    }

    fn reset_node_selection(&mut self) {
        self.node_completion_input.clear();
        self.node_completion_cursor = 0;
        self.is_node_completion_mode = false;
    }

    fn enter_completing_read_mode(&mut self) {
        self.view = View::NodeSelect;
        self.node_completion_input.clear();
        self.node_completion_cursor = 0;
        self.is_node_completion_mode = true;
        self.update_node_suggestions();
    }
}

fn draw_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.size());

    match app.view {
        View::TopicList => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else {
                render_topic_list(
                    frame,
                    chunks[0],
                    &app.topics,
                    app.selected_topic,
                    &app.current_node,
                    &app.theme,
                );
            }
        }
        View::TopicDetail => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else if let Some(ref topic) = app.current_topic {
                if app.show_replies {
                    let area = chunks[0];
                    // Use vertical layout when terminal is narrow (< 100 columns)
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
                        app.topic_scroll,
                        &app.theme,
                    );
                    render_replies(
                        frame,
                        split_chunks[1],
                        &app.topic_replies,
                        &mut app.replies_list_state,
                        &app.theme,
                    );
                } else {
                    render_topic_detail(frame, chunks[0], topic, app.topic_scroll, &app.theme);
                }
            }
        }
        View::Notifications => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else {
                render_notifications(
                    frame,
                    chunks[0],
                    &app.notifications,
                    app.selected_notification,
                    &app.theme,
                );
            }
        }
        View::Profile => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else if let Some(ref profile) = app.profile {
                render_profile(frame, chunks[0], profile, &app.theme);
            }
        }
        View::Help => {
            render_help(frame, chunks[0], &app.theme);
        }
        View::NodeSelect => {
            render_node_select(
                frame,
                chunks[0],
                &app.favorite_nodes,
                app.selected_node,
                &app.current_node,
                &app.node_completion_input,
                app.node_completion_cursor,
                app.is_node_completion_mode,
                &app.theme,
            );
        }
        View::TokenInput => {
            render_token_input(
                frame,
                chunks[0],
                &app.token_input,
                app.token_cursor,
                &app.theme,
            );
        }
        View::ReplyInput => {
            render_reply_input(
                frame,
                chunks[0],
                &app.reply_input,
                app.reply_cursor,
                &app.theme,
            );
        }
    }

    render_status_bar(frame, chunks[1], &app.status_message, &app.theme);
}

async fn run_app(terminal: &mut Terminal<impl Backend>, client: V2exClient) -> Result<()> {
    let mut app = App::new();

    // Load initial topics
    app.load_topics(&client, false).await;

    loop {
        terminal.draw(|frame| draw_ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Handle ReplyInput view separately
                if app.view == View::ReplyInput {
                    match key.code {
                        KeyCode::Char(c) if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            app.reply_input.insert(app.reply_cursor, c);
                            app.reply_cursor += 1;
                        }
                        KeyCode::Backspace => {
                            if app.reply_cursor > 0 {
                                app.reply_input.remove(app.reply_cursor - 1);
                                app.reply_cursor -= 1;
                            }
                        }
                        KeyCode::Delete => {
                            if app.reply_cursor < app.reply_input.chars().count() {
                                app.reply_input.remove(app.reply_cursor);
                            }
                        }
                        KeyCode::Left => {
                            if app.reply_cursor > 0 {
                                app.reply_cursor -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.reply_cursor < app.reply_input.chars().count() {
                                app.reply_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Submit reply
                            if let Some(ref topic) = app.current_topic {
                                let topic_id = topic.id;
                                let content = app.reply_input.clone();
                                if !content.trim().is_empty() {
                                    app.loading = true;
                                    app.status_message = "Posting reply...".to_string();
                                    match client.create_reply(topic_id, content).await {
                                        Ok(reply) => {
                                            app.status_message = "Reply posted successfully".to_string();
                                            // Add the new reply to the list
                                            app.topic_replies.push(reply);
                                            // Return to topic detail view
                                            app.view = View::TopicDetail;
                                            app.reply_input.clear();
                                            app.reply_cursor = 0;
                                            // Refresh replies to ensure we have the latest
                                            app.load_topic_replies(&client, topic_id, false).await;
                                        }
                                        Err(e) => {
                                            app.error = Some(format!("Failed to post reply: {}", e));
                                            app.status_message = "Failed to post reply".to_string();
                                        }
                                    }
                                    app.loading = false;
                                } else {
                                    app.status_message = "Reply cannot be empty".to_string();
                                }
                            }
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Cancel reply input
                            app.view = View::TopicDetail;
                            app.reply_input.clear();
                            app.reply_cursor = 0;
                            app.status_message = "Reply cancelled".to_string();
                        }
                        KeyCode::Esc => {
                            // Cancel reply input
                            app.view = View::TopicDetail;
                            app.reply_input.clear();
                            app.reply_cursor = 0;
                            app.status_message = "Reply cancelled".to_string();
                        }
                        _ => {}
                    }
                    continue; // Skip the main key handling
                }
                
                match key.code {
                    KeyCode::Char('q') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('q');
                        } else {
                            match app.view {
                                View::TopicList => break,
                                View::NodeSelect => {
                                    app.view = View::TopicList;
                                }
                                _ => {
                                    app.view = View::TopicList;
                                    app.error = None;
                                }
                            }
                        }
                    }
                    KeyCode::Esc => match app.view {
                        View::TopicList => break,
                        View::NodeSelect => {
                            app.view = View::TopicList;
                        }
                        _ => {
                            app.view = View::TopicList;
                            app.error = None;
                        }
                    },
                    KeyCode::Char('?') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('?');
                        } else {
                            app.view = View::Help;
                        }
                    }
                    KeyCode::Char('h') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('h');
                        } else {
                            match app.view {
                                View::NodeSelect => {
                                    app.view = View::TopicList;
                                }
                                _ => {
                                    if app.view != View::TopicList {
                                        app.view = View::TopicList;
                                        app.error = None;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Left => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.move_node_cursor_left();
                        } else {
                            match app.view {
                                View::NodeSelect => {
                                    app.view = View::TopicList;
                                }
                                _ => {
                                    if app.view != View::TopicList {
                                        app.view = View::TopicList;
                                        app.error = None;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('n')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        if app.view == View::NodeSelect {
                            app.next_node();
                        }
                    }
                    KeyCode::Char('n') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('n');
                        } else {
                            match app.view {
                                View::TopicList => app.next_topic(),
                                View::Notifications => app.next_notification(),
                                View::NodeSelect => app.next_node(),
                                View::TopicDetail => {
                                    if app.show_replies && !app.topic_replies.is_empty() {
                                        app.next_reply();
                                    } else {
                                        app.scroll_topic_down();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Down => {
                        match app.view {
                            View::TopicList => app.next_topic(),
                            View::Notifications => app.next_notification(),
                            View::NodeSelect => {
                                if app.is_node_completion_mode {
                                    // Do nothing in manual mode
                                } else {
                                    app.next_node();
                                }
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    app.next_reply();
                                } else {
                                    app.scroll_topic_down();
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('p')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        if app.view == View::NodeSelect {
                            app.previous_node();
                        }
                    }
                    KeyCode::Char('p') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('p');
                        } else {
                            match app.view {
                                View::TopicList => app.previous_topic(),
                                View::Notifications => app.previous_notification(),
                                View::NodeSelect => app.previous_node(),
                                View::TopicDetail => {
                                    if app.show_replies && !app.topic_replies.is_empty() {
                                        app.previous_reply();
                                    } else {
                                        app.scroll_topic_up();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Up => {
                        match app.view {
                            View::TopicList => app.previous_topic(),
                            View::Notifications => app.previous_notification(),
                            View::NodeSelect => {
                                if app.is_node_completion_mode {
                                    // Do nothing in manual mode
                                } else {
                                    app.previous_node();
                                }
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    app.previous_reply();
                                } else {
                                    app.scroll_topic_up();
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('l') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('l');
                        } else {
                            match app.view {
                                View::TopicList => {
                                    if let Some(topic) = app.topics.get(app.selected_topic) {
                                        let topic_id = topic.id;
                                        app.view = View::TopicDetail;
                                        app.show_replies = true;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id, false).await;
                                    }
                                }
                                View::Notifications => {
                                    if let Some(notification) =
                                        app.notifications.get(app.selected_notification)
                                    {
                                        let topic_id = notification.extract_topic_id();
                                        let reply_id = notification.extract_reply_id();

                                        if let Some(topic_id) = topic_id {
                                            app.view = View::TopicDetail;
                                            app.show_replies = true;
                                            app.load_topic_detail(&client, topic_id).await;
                                            app.load_topic_replies(&client, topic_id, false).await;

                                            // Update status message
                                            if let Some(reply_id) = reply_id {
                                                app.status_message = format!(
                                                    "Jumping to topic {} (reply #{})",
                                                    topic_id, reply_id
                                                );
                                            } else {
                                                app.status_message =
                                                    format!("Jumping to topic {}", topic_id);
                                            }
                                        } else {
                                            app.status_message =
                                                "No topic link found in this notification"
                                                    .to_string();
                                        }
                                    }
                                }
                                View::NodeSelect => {
                                    app.select_current_node();
                                    app.reset_node_selection();
                                    app.view = View::TopicList;
                                    app.load_topics(&client, false).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Right => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.move_node_cursor_right();
                        } else {
                            match app.view {
                                View::TopicList => {
                                    if let Some(topic) = app.topics.get(app.selected_topic) {
                                        let topic_id = topic.id;
                                        app.view = View::TopicDetail;
                                        app.show_replies = true;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id, false).await;
                                    }
                                }
                                View::Notifications => {
                                    if let Some(notification) =
                                        app.notifications.get(app.selected_notification)
                                    {
                                        let topic_id = notification.extract_topic_id();
                                        let reply_id = notification.extract_reply_id();

                                        if let Some(topic_id) = topic_id {
                                            app.view = View::TopicDetail;
                                            app.show_replies = true;
                                            app.load_topic_detail(&client, topic_id).await;
                                            app.load_topic_replies(&client, topic_id, false).await;

                                            // Update status message
                                            if let Some(reply_id) = reply_id {
                                                app.status_message = format!(
                                                    "Jumping to topic {} (reply #{})",
                                                    topic_id, reply_id
                                                );
                                            } else {
                                                app.status_message =
                                                    format!("Jumping to topic {}", topic_id);
                                            }
                                        } else {
                                            app.status_message =
                                                "No topic link found in this notification"
                                                    .to_string();
                                        }
                                    }
                                }
                                View::NodeSelect => {
                                    app.select_current_node();
                                    app.reset_node_selection();
                                    app.view = View::TopicList;
                                    app.load_topics(&client, false).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Enter => {
                        match app.view {
                            View::TopicList => {
                                if let Some(topic) = app.topics.get(app.selected_topic) {
                                    let topic_id = topic.id;
                                    app.view = View::TopicDetail;
                                    app.show_replies = true;
                                    app.load_topic_detail(&client, topic_id).await;
                                    app.load_topic_replies(&client, topic_id, false).await;
                                }
                            }
                            View::Notifications => {
                                if let Some(notification) =
                                    app.notifications.get(app.selected_notification)
                                {
                                    let topic_id = notification.extract_topic_id();
                                    let reply_id = notification.extract_reply_id();

                                    if let Some(topic_id) = topic_id {
                                        app.view = View::TopicDetail;
                                        app.show_replies = true;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id, false).await;

                                        // Update status message
                                        if let Some(reply_id) = reply_id {
                                            app.status_message = format!(
                                                "Jumping to topic {} (reply #{})",
                                                topic_id, reply_id
                                            );
                                        } else {
                                            app.status_message =
                                                format!("Jumping to topic {}", topic_id);
                                        }
                                    } else {
                                        app.status_message =
                                            "No topic link found in this notification".to_string();
                                    }
                                }
                            }
                            View::NodeSelect => {
                                app.select_current_node();
                                app.reset_node_selection();
                                app.view = View::TopicList;
                                app.load_topics(&client, false).await;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('r') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('r');
                        } else {
                            match app.view {
                                View::TopicList => app.load_topics(&client, false).await,
                                View::TopicDetail => {
                                    if let Some(ref topic) = app.current_topic {
                                        let topic_id = topic.id;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id, false).await;
                                    }
                                }
                                View::Notifications => app.load_notifications(&client).await,
                                View::Profile => app.load_profile(&client).await,
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('a') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('a');
                        } else if app.view == View::TopicDetail {
                            // Enter reply input mode
                            app.view = View::ReplyInput;
                            app.reply_input.clear();
                            app.reply_cursor = 0;
                            app.status_message = "Type your reply. Press Enter to submit, Ctrl+C to cancel".to_string();
                        }
                    }
                    KeyCode::Char('m') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('m');
                        } else {
                            app.view = View::Notifications;
                            app.load_notifications(&client).await;
                        }
                    }
                    KeyCode::Char('u') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('u');
                        } else {
                            app.view = View::Profile;
                            app.load_profile(&client).await;
                        }
                    }
                    KeyCode::Char('s') => {
                        match app.view {
                            View::NodeSelect => {
                                if app.is_node_completion_mode {
                                    // In manual mode, insert 's' as character
                                    app.insert_node_char('s');
                                } else {
                                    // Already in node select, toggle manual mode
                                    app.toggle_node_completion_mode();
                                }
                            }
                            _ => {
                                // Directly enter completing-read mode
                                app.enter_completing_read_mode();
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if app.view == View::NodeSelect {
                            app.toggle_node_completion_mode();
                        }
                    }
                    KeyCode::Char('t') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('t');
                        } else {
                            match app.view {
                                View::TopicList => {
                                    if let Some(topic) = app.topics.get(app.selected_topic) {
                                        let topic_id = topic.id;
                                        app.view = View::TopicDetail;
                                        app.show_replies = true;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id, false).await;
                                    }
                                }
                                View::TopicDetail => {
                                    app.show_replies = !app.show_replies;
                                    app.reset_scroll();
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('o');
                        } else {
                            match app.view {
                                View::TopicDetail => {
                                    if let Some(ref topic) = app.current_topic {
                                        // Check if we're in replies view and a reply is selected
                                        if app.show_replies && !app.topic_replies.is_empty() {
                                            if let Some(reply) =
                                                app.topic_replies.get(app.selected_reply)
                                            {
                                                let url = format!(
                                                    "https://www.v2ex.com/t/{}#r_{}",
                                                    topic.id, reply.id
                                                );
                                                match webbrowser::open(&url) {
                                                    Ok(_) => {
                                                        app.status_message = format!(
                                                            "Opened topic {} (reply #{}) in browser",
                                                            topic.id, reply.id
                                                        );
                                                    }
                                                    Err(e) => {
                                                        app.error = Some(format!(
                                                            "Failed to open browser: {}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            } else {
                                                // Fallback to topic URL if selected reply is out of bounds
                                                let url =
                                                    format!("https://www.v2ex.com/t/{}", topic.id);
                                                match webbrowser::open(&url) {
                                                    Ok(_) => {
                                                        app.status_message = format!(
                                                            "Opened topic {} in browser",
                                                            topic.id
                                                        );
                                                    }
                                                    Err(e) => {
                                                        app.error = Some(format!(
                                                            "Failed to open browser: {}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Not in replies view or no replies, open topic URL
                                            let url =
                                                format!("https://www.v2ex.com/t/{}", topic.id);
                                            match webbrowser::open(&url) {
                                                Ok(_) => {
                                                    app.status_message = format!(
                                                        "Opened topic {} in browser",
                                                        topic.id
                                                    );
                                                }
                                                Err(e) => {
                                                    app.error = Some(format!(
                                                        "Failed to open browser: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                                View::TopicList => {
                                    if let Some(topic) = app.topics.get(app.selected_topic) {
                                        let url = format!("https://www.v2ex.com/t/{}", topic.id);
                                        match webbrowser::open(&url) {
                                            Ok(_) => {
                                                app.status_message =
                                                    format!("Opened topic {} in browser", topic.id);
                                            }
                                            Err(e) => {
                                                app.error =
                                                    Some(format!("Failed to open browser: {}", e));
                                            }
                                        }
                                    }
                                }
                                View::Notifications => {
                                    if let Some(notification) =
                                        app.notifications.get(app.selected_notification)
                                    {
                                        if let Some(topic_id) = notification.extract_topic_id() {
                                            let url =
                                                format!("https://www.v2ex.com/t/{}", topic_id);
                                            match webbrowser::open(&url) {
                                                Ok(_) => {
                                                    app.status_message = format!(
                                                        "Opened topic {} in browser",
                                                        topic_id
                                                    );
                                                }
                                                Err(e) => {
                                                    app.error = Some(format!(
                                                        "Failed to open browser: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        } else {
                                            app.status_message =
                                                "No topic link in this notification".to_string();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('N') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('N');
                        } else if app.view == View::TopicDetail {
                            app.switch_to_next_topic(&client).await;
                        }
                    }
                    KeyCode::Char('P') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('P');
                        } else if app.view == View::TopicDetail {
                            app.switch_to_previous_topic(&client).await;
                        }
                    }
                    KeyCode::Char('1') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('1');
                        } else {
                            app.switch_node("python");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('2') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('2');
                        } else {
                            app.switch_node("programmer");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('3') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('3');
                        } else {
                            app.switch_node("share");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('4') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('4');
                        } else {
                            app.switch_node("create");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('5') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('5');
                        } else {
                            app.switch_node("jobs");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('6') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('6');
                        } else {
                            app.switch_node("go");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('7') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('7');
                        } else {
                            app.switch_node("rust");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('8') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('8');
                        } else {
                            app.switch_node("javascript");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::Char('9') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('9');
                        } else {
                            app.switch_node("linux");
                            app.load_topics(&client, false).await;
                        }
                    }
                    KeyCode::PageDown => {
                        match app.view {
                            View::TopicList => {
                                app.page += 1;
                                app.load_topics(&client, true).await;
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    // Move 5 replies forward
                                    app.selected_reply =
                                        (app.selected_reply + 5).min(app.topic_replies.len() - 1);
                                } else {
                                    app.topic_scroll += 15; // Scroll 15 lines
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::PageUp => {
                        match app.view {
                            View::TopicList => {
                                if app.page > 1 {
                                    app.page -= 1;
                                    app.load_topics(&client, false).await;
                                }
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    // Move 5 replies backward
                                    if app.selected_reply >= 5 {
                                        app.selected_reply -= 5;
                                    } else {
                                        app.selected_reply = 0;
                                    }
                                } else if app.topic_scroll >= 15 {
                                    app.topic_scroll -= 15;
                                } else {
                                    app.topic_scroll = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('+') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('+');
                        } else if app.view == View::TopicList {
                            app.page += 1;
                            app.load_topics(&client, true).await;
                        } else if app.view == View::TopicDetail && app.show_replies {
                            if let Some(ref topic) = app.current_topic {
                                app.load_topic_replies(&client, topic.id, true).await;
                            }
                        }
                    }
                    KeyCode::Char('<') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('<');
                        } else {
                            match app.view {
                                View::TopicList => app.selected_topic = 0,
                                View::Notifications => app.selected_notification = 0,
                                View::TopicDetail => {
                                    if app.show_replies && !app.topic_replies.is_empty() {
                                        app.selected_reply = 0;
                                        app.replies_list_state.select(Some(0));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char('>') => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char('>');
                        } else {
                            match app.view {
                                View::TopicList => {
                                    if !app.topics.is_empty() {
                                        app.selected_topic = app.topics.len() - 1;
                                    }
                                }
                                View::Notifications => {
                                    if !app.notifications.is_empty() {
                                        app.selected_notification = app.notifications.len() - 1;
                                    }
                                }
                                View::TopicDetail => {
                                    if app.show_replies && !app.topic_replies.is_empty() {
                                        app.selected_reply = app.topic_replies.len() - 1;
                                        app.replies_list_state.select(Some(app.selected_reply));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Char(ch) => {
                        // Handle character input for manual node mode
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.insert_node_char(ch);
                        }
                    }
                    KeyCode::Backspace => {
                        if app.view == View::NodeSelect && app.is_node_completion_mode {
                            app.delete_node_char();
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("v2ex-tui - A terminal UI viewer for V2EX");
    println!();
    println!("Usage: v2ex-tui [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help     Print help information");
    println!("  -v, --version  Print version information");
    println!();
    println!("Configuration:");
    println!("  Token file: ~/.config/v2ex/token.txt");
    println!();
    println!("  Get your Personal Access Token from:");
    println!("  https://www.v2ex.com/settings/tokens");
    println!();
    println!("Keyboard Shortcuts (Emacs/dired style):");
    println!("  n/p or ↓/↑     Move down/up (next/previous)");
    println!("  h/← or l/→     Navigate back/forward");
    println!("  Enter/t        Open selected topic/notification");
    println!("  r              Refresh");
    println!("  m              Notifications (messages)");
    println!("  u              Profile (user)");
    println!("  s              Select node from menu (Tab: manual input)");
    println!("  1-9            Quick switch nodes (1:python, etc.)");
    println!(
        "  t              Open topic / Toggle replies view
  o              Open current topic in browser"
    );
    println!("  +              Load more topics");
    println!("  PageUp/Down    Load previous/next page of topics");
    println!("  N/P            Next/previous topic in detail view");
    println!("  </>            Go to first/last item");
    println!("  ?              Help");
    println!("  q/Esc          Quit");
}

fn print_version() {
    println!("v2ex-tui 0.1.0");
}

async fn run_token_input(terminal: &mut Terminal<impl Backend>) -> Result<Option<String>> {
    let mut app = App::new();
    app.view = View::TokenInput;
    app.status_message = "Enter your V2EX token".to_string();

    loop {
        terminal.draw(|frame| draw_ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        return Ok(None);
                    }
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        if !app.token_input.trim().is_empty() {
                            // Try to save the token
                            match app.save_token() {
                                Ok(_) => {
                                    return Ok(Some(app.token_input.trim().to_string()));
                                }
                                Err(e) => {
                                    app.status_message = format!("Error saving token: {}", e);
                                }
                            }
                        } else {
                            app.status_message = "Token cannot be empty".to_string();
                        }
                    }
                    KeyCode::Char(ch) => {
                        app.insert_token_char(ch);
                    }
                    KeyCode::Backspace => {
                        app.delete_token_char();
                    }
                    KeyCode::Left => {
                        app.move_token_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_token_cursor_right();
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    // Try to load token
    let token = match V2exClient::load_token() {
        Ok(t) => t,
        Err(_) => {
            // Token not found, setup terminal and show token input view
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Run token input UI
            let token_result = run_token_input(&mut terminal).await;

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            match token_result? {
                Some(t) => t,
                None => {
                    println!("Token input cancelled.");
                    std::process::exit(1);
                }
            }
        }
    };

    let client = V2exClient::new(token.clone());

    // Test API connection before starting TUI
    match client.get_member().await {
        Ok(member) => {
            println!("Connected to V2EX as: {}", member.username);
        }
        Err(e) => {
            // Token is invalid, remove it
            if let Ok(config_dir) = V2exClient::config_dir() {
                let token_path = config_dir.join("token.txt");
                let _ = std::fs::remove_file(&token_path);
            }
            eprintln!("Error: Failed to connect to V2EX API: {}", e);
            eprintln!(
                "The token has been removed. Please run the application again with a valid token."
            );
            std::process::exit(1);
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, client).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
