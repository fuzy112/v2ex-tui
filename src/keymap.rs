use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::api::V2exClient;
use crate::app::{App, View};

/// Trait for key mappings
pub trait KeyMap {
    fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> impl std::future::Future<Output = Result<bool>>;
}

/// Link selection mode key mapping
pub struct LinkSelectionKeyMap;

impl LinkSelectionKeyMap {
    pub fn new() -> Self {
        Self
    }

    async fn handle_link_mode_char(
        &self,
        app: &mut App,
        ch: char,
    ) -> Result<bool> {
        if !app.topic_state.link_input_state.is_active {
            return Ok(false);
        }

        let (input, timeout_reset, valid_input) = app.topic_state.handle_link_mode_key(ch);

        // Check if input is invalid (not a home row letter)
        if !valid_input {
            app.ui_state.error = Some(format!(
                "Invalid key '{}' - only home row letters (a/o/e/u/i/d/h/t/n/s) are allowed",
                ch
            ));
            app.topic_state.exit_link_selection_mode();
            return Ok(false);
        }

        if timeout_reset {
            app.ui_state.status_message =
                format!("Link mode: input reset (timeout). Current: '{}'", input);
        } else {
            app.ui_state.status_message = format!("Link mode: input '{}'", input);
        }

        // Prefix matching with exact match detection
        let matches = app.topic_state.find_links_by_prefix(&input);

        // Check for exact match (input length equals shortcut length)
        let exact_match = matches
            .iter()
            .find(|link| link.shortcut.len() == input.len());

        if let Some(link) = exact_match {
            // Exact match found - open the link
            match crate::browser::Browser::open_url(&link.url) {
                Ok(result) => {
                    app.ui_state.status_message =
                        format!("Opening link {}: {}", link.shortcut, result);
                }
                Err(e) => {
                    app.ui_state.error =
                        Some(format!("Failed to open link {}: {}", link.shortcut, e));
                }
            }
            app.topic_state.exit_link_selection_mode();
        } else if !matches.is_empty() {
            // Multiple prefix matches - show feedback
            let shortcuts: Vec<&str> = matches.iter().map(|l| l.shortcut.as_str()).collect();
            app.ui_state.status_message = format!(
                "Link mode: input '{}', matches: {}",
                input,
                shortcuts.join(", ")
            );
        }
        // matches.len() == 0: silent ignore (user confirmed)

        Ok(false)
    }
}

impl KeyMap for LinkSelectionKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        _client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                app.topic_state.exit_link_selection_mode();
                Ok(true)
            }
            KeyCode::Esc => {
                app.topic_state.exit_link_selection_mode();
                app.ui_state.status_message = "Link selection cancelled".to_string();
                Ok(false)
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.topic_state.exit_link_selection_mode();
                app.ui_state.status_message = "Link selection cancelled".to_string();
                Ok(false)
            }
            // Home row keys for link selection
            KeyCode::Char('a') => self.handle_link_mode_char(app, 'a').await,
            KeyCode::Char('o') => self.handle_link_mode_char(app, 'o').await,
            KeyCode::Char('e') => self.handle_link_mode_char(app, 'e').await,
            KeyCode::Char('u') => self.handle_link_mode_char(app, 'u').await,
            KeyCode::Char('i') => self.handle_link_mode_char(app, 'i').await,
            KeyCode::Char('d') => self.handle_link_mode_char(app, 'd').await,
            KeyCode::Char('h') => self.handle_link_mode_char(app, 'h').await,
            KeyCode::Char('t') => self.handle_link_mode_char(app, 't').await,
            KeyCode::Char('n') => self.handle_link_mode_char(app, 'n').await,
            KeyCode::Char('s') => self.handle_link_mode_char(app, 's').await,
            KeyCode::Char(ch) => {
                // Any other character key is invalid in link selection mode
                app.ui_state.status_message = format!(
                    "Invalid key '{}' - only home row letters (a/o/e/u/i/d/h/t/n/s) are allowed",
                    ch
                );
                app.topic_state.exit_link_selection_mode();
                Ok(false)
            }
            _ => {
                // Ignore non-character keys in link selection mode
                Ok(false)
            }
        }
    }
}

/// Topic list view key mapping
pub struct TopicListKeyMap;

impl TopicListKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for TopicListKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => Ok(true),
            KeyCode::Char('?') => {
                app.previous_view = Some(app.view);
                app.view = View::Help;
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // Already in topic list, no-op
                Ok(false)
            }
            KeyCode::Char('n') => {
                app.topic_state.next_topic();
                Ok(false)
            }
            KeyCode::Down => {
                app.topic_state.next_topic();
                Ok(false)
            }
            KeyCode::Char('p') => {
                app.topic_state.previous_topic();
                Ok(false)
            }
            KeyCode::Up => {
                app.topic_state.previous_topic();
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                    let topic_id = topic.id;
                    app.view = View::TopicDetail;
                    app.topic_state.show_replies = true;
                    app.load_topic_detail(client, topic_id).await;
                    app.load_topic_replies(client, topic_id, false).await;
                }
                Ok(false)
            }
            KeyCode::Char('r') => {
                app.open_selected_topic_in_browser();
                Ok(false)
            }
            KeyCode::Char('g') => {
                app.load_topics(client, false).await;
                Ok(false)
            }
            KeyCode::Char('a') => {
                app.previous_view = Some(app.view);
                app.view = View::Aggregate;
                app.load_aggregate(client).await;
                Ok(false)
            }
            KeyCode::Char('m') => {
                app.previous_view = Some(app.view);
                app.view = View::Notifications;
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('u') => {
                app.previous_view = Some(app.view);
                app.view = View::Profile;
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('s') => {
                app.previous_view = Some(app.view);
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                Ok(false)
            }
            KeyCode::Char('t') => {
                if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                    let topic_id = topic.id;
                    app.view = View::TopicDetail;
                    app.topic_state.show_replies = true;
                    app.load_topic_detail(client, topic_id).await;
                    app.load_topic_replies(client, topic_id, false).await;
                }
                Ok(false)
            }
            KeyCode::Char('o') => {
                app.open_selected_topic_in_browser();
                Ok(false)
            }
            KeyCode::PageDown => {
                app.node_state.page += 1;
                app.load_topics(client, true).await;
                Ok(false)
            }
            KeyCode::PageUp => {
                if app.node_state.page > 1 {
                    app.node_state.page -= 1;
                    app.load_topics(client, false).await;
                }
                Ok(false)
            }
            KeyCode::Char('+') => {
                app.node_state.page += 1;
                app.load_topics(client, true).await;
                Ok(false)
            }
            KeyCode::Char('<') => {
                app.topic_state.selected = 0;
                Ok(false)
            }
            KeyCode::Char('>') => {
                if !app.topic_state.topics.is_empty() {
                    app.topic_state.selected = app.topic_state.topics.len() - 1;
                }
                Ok(false)
            }
            KeyCode::Char(ch) => {
                // Handle number keys for quick node switching
                match ch {
                    '1' => {
                        app.node_state.switch_node("python");
                        app.load_topics(client, false).await;
                    }
                    '2' => {
                        app.node_state.switch_node("programmer");
                        app.load_topics(client, false).await;
                    }
                    '3' => {
                        app.node_state.switch_node("share");
                        app.load_topics(client, false).await;
                    }
                    '4' => {
                        app.node_state.switch_node("create");
                        app.load_topics(client, false).await;
                    }
                    '5' => {
                        app.node_state.switch_node("jobs");
                        app.load_topics(client, false).await;
                    }
                    '6' => {
                        app.node_state.switch_node("go");
                        app.load_topics(client, false).await;
                    }
                    '7' => {
                        app.node_state.switch_node("rust");
                        app.load_topics(client, false).await;
                    }
                    '8' => {
                        app.node_state.switch_node("javascript");
                        app.load_topics(client, false).await;
                    }
                    '9' => {
                        app.node_state.switch_node("linux");
                        app.load_topics(client, false).await;
                    }
                    _ => {}
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Topic detail view key mapping
pub struct TopicDetailKeyMap;

impl TopicDetailKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for TopicDetailKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Esc => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('?') => {
                app.previous_view = Some(app.view);
                app.view = View::Help;
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.view = View::TopicList;
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('n') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.next_reply(app.terminal_width);
                } else {
                    app.topic_state.scroll_down();
                }
                Ok(false)
            }
            KeyCode::Down => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.next_reply(app.terminal_width);
                } else {
                    app.topic_state.scroll_down();
                }
                Ok(false)
            }
            KeyCode::Char('p') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.previous_reply(app.terminal_width);
                } else {
                    app.topic_state.scroll_up();
                }
                Ok(false)
            }
            KeyCode::Up => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.previous_reply(app.terminal_width);
                } else {
                    app.topic_state.scroll_up();
                }
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Right => {
                // Already in topic detail, no-op
                Ok(false)
            }
            KeyCode::Enter => {
                // Already in topic detail, no-op
                Ok(false)
            }
            KeyCode::Char('r') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.open_selected_reply_in_browser();
                } else {
                    app.open_current_topic_in_browser();
                }
                Ok(false)
            }
            KeyCode::Char('f') => {
                app.topic_state.enter_link_selection_mode(app.terminal_width);
                app.ui_state.status_message =
                    "Link mode: press a/o/e/u/i/d/h/t/n/s (home row), Ctrl+g to cancel".to_string();
                Ok(false)
            }
            KeyCode::Char('g') => {
                if let Some(ref topic) = app.topic_state.current {
                    let topic_id = topic.id;
                    app.load_topic_detail(client, topic_id).await;
                    app.load_topic_replies(client, topic_id, false).await;
                }
                Ok(false)
            }
            KeyCode::Char('a') => {
                app.previous_view = Some(app.view);
                app.view = View::Aggregate;
                app.load_aggregate(client).await;
                Ok(false)
            }
            KeyCode::Char('m') => {
                app.previous_view = Some(app.view);
                app.view = View::Notifications;
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('u') => {
                app.previous_view = Some(app.view);
                app.view = View::Profile;
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('s') => {
                app.previous_view = Some(app.view);
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                Ok(false)
            }
            KeyCode::Char('t') => {
                app.topic_state.show_replies = !app.topic_state.show_replies;
                app.topic_state.reset_scroll();
                Ok(false)
            }
            KeyCode::Char('o') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.open_selected_reply_in_browser();
                } else {
                    app.open_current_topic_in_browser();
                }
                Ok(false)
            }
            KeyCode::Char('N') => {
                app.switch_to_next_topic(client).await;
                Ok(false)
            }
            KeyCode::Char('P') => {
                app.switch_to_previous_topic(client).await;
                Ok(false)
            }
            KeyCode::PageDown => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.selected_reply =
                        (app.topic_state.selected_reply + 5).min(app.topic_state.replies.len() - 1);
                } else {
                    app.topic_state.scroll += 15;
                }
                Ok(false)
            }
            KeyCode::PageUp => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    if app.topic_state.selected_reply >= 5 {
                        app.topic_state.selected_reply -= 5;
                    } else {
                        app.topic_state.selected_reply = 0;
                    }
                } else if app.topic_state.scroll >= 15 {
                    app.topic_state.scroll -= 15;
                } else {
                    app.topic_state.scroll = 0;
                }
                Ok(false)
            }
            KeyCode::Char('+') => {
                if app.topic_state.show_replies {
                    if let Some(ref topic) = app.topic_state.current {
                        app.load_topic_replies(client, topic.id, true).await;
                    }
                }
                Ok(false)
            }
            KeyCode::Char('<') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.selected_reply = 0;
                    app.topic_state.replies_list_state.select(Some(0));
                }
                Ok(false)
            }
            KeyCode::Char('>') => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.selected_reply = app.topic_state.replies.len() - 1;
                    app.topic_state
                        .replies_list_state
                        .select(Some(app.topic_state.selected_reply));
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Notifications view key mapping
pub struct NotificationsKeyMap;

impl NotificationsKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for NotificationsKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Esc => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('?') => {
                app.previous_view = Some(app.view);
                app.view = View::Help;
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.view = View::TopicList;
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('n') => {
                app.notification_state.next();
                Ok(false)
            }
            KeyCode::Down => {
                app.notification_state.next();
                Ok(false)
            }
            KeyCode::Char('p') => {
                app.notification_state.previous();
                Ok(false)
            }
            KeyCode::Up => {
                app.notification_state.previous();
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(notification) = app
                    .notification_state
                    .notifications
                    .get(app.notification_state.selected)
                {
                    let topic_id = notification.extract_topic_id();
                    let reply_id = notification.extract_reply_id();

                    if let Some(topic_id) = topic_id {
                        app.previous_view = Some(app.view);
                        app.view = View::TopicDetail;
                        app.topic_state.show_replies = true;
                        app.load_topic_detail(client, topic_id).await;
                        app.load_topic_replies(client, topic_id, false).await;

                        if let Some(reply_id) = reply_id {
                            app.ui_state.status_message =
                                format!("Jumping to topic {} (reply #{})", topic_id, reply_id);
                        } else {
                            app.ui_state.status_message = format!("Jumping to topic {}", topic_id);
                        }
                    } else {
                        app.ui_state.status_message =
                            "No topic link found in this notification".to_string();
                    }
                }
                Ok(false)
            }
            KeyCode::Char('r') => {
                app.open_notification_in_browser();
                Ok(false)
            }
            KeyCode::Char('g') => {
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('a') => {
                app.previous_view = Some(app.view);
                app.view = View::Aggregate;
                app.load_aggregate(client).await;
                Ok(false)
            }
            KeyCode::Char('m') => {
                // Already in notifications, refresh
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('u') => {
                app.previous_view = Some(app.view);
                app.view = View::Profile;
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('s') => {
                app.previous_view = Some(app.view);
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                Ok(false)
            }
            KeyCode::Char('o') => {
                app.open_notification_in_browser();
                Ok(false)
            }
            KeyCode::Char('<') => {
                app.notification_state.selected = 0;
                Ok(false)
            }
            KeyCode::Char('>') => {
                if !app.notification_state.notifications.is_empty() {
                    app.notification_state.selected =
                        app.notification_state.notifications.len() - 1;
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Profile view key mapping
pub struct ProfileKeyMap;

impl ProfileKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for ProfileKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Esc => {
                app.view = app.previous_view.unwrap_or(View::TopicList);
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('?') => {
                app.previous_view = Some(app.view);
                app.view = View::Help;
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.view = View::TopicList;
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('g') => {
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('a') => {
                app.previous_view = Some(app.view);
                app.view = View::Aggregate;
                app.load_aggregate(client).await;
                Ok(false)
            }
            KeyCode::Char('m') => {
                app.previous_view = Some(app.view);
                app.view = View::Notifications;
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('u') => {
                // Already in profile, refresh
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('s') => {
                app.previous_view = Some(app.view);
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Help view key mapping
pub struct HelpKeyMap;

impl HelpKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for HelpKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        _key: KeyEvent,
        _client: &V2exClient,
    ) -> Result<bool> {
        // Any key exits help view
        app.view = app.previous_view.unwrap_or(View::TopicList);
        Ok(false)
    }
}

/// Node select view key mapping
pub struct NodeSelectKeyMap;

impl NodeSelectKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for NodeSelectKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('q');
                } else {
                    app.view = View::TopicList;
                }
                Ok(false)
            }
            KeyCode::Esc => {
                app.view = View::TopicList;
                Ok(false)
            }
            KeyCode::Char('?') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('?');
                } else {
                    app.previous_view = Some(app.view);
                    app.view = View::Help;
                }
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('h');
                } else {
                    app.view = View::TopicList;
                }
                Ok(false)
            }
            KeyCode::Char('n') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.node_state.next_node();
                } else if app.node_state.is_completion_mode {
                    app.node_state.insert_char('n');
                } else {
                    app.node_state.next_node();
                }
                Ok(false)
            }
            KeyCode::Down => {
                if !app.node_state.is_completion_mode {
                    app.node_state.next_node();
                }
                Ok(false)
            }
            KeyCode::Char('p') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.node_state.previous_node();
                } else if app.node_state.is_completion_mode {
                    app.node_state.insert_char('p');
                } else {
                    app.node_state.previous_node();
                }
                Ok(false)
            }
            KeyCode::Up => {
                if !app.node_state.is_completion_mode {
                    app.node_state.previous_node();
                }
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('l');
                } else {
                    app.node_state.select_current_node();
                    app.node_state.reset_selection();
                    app.view = View::TopicList;
                    app.load_topics(client, false).await;
                }
                Ok(false)
            }
            KeyCode::Char('g') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('g');
                }
                Ok(false)
            }
            KeyCode::Char('a') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('a');
                }
                Ok(false)
            }
            KeyCode::Char('m') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('m');
                } else {
                    app.previous_view = Some(app.view);
                    app.view = View::Notifications;
                    app.load_notifications(client).await;
                }
                Ok(false)
            }
            KeyCode::Char('u') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('u');
                } else {
                    app.previous_view = Some(app.view);
                    app.view = View::Profile;
                    app.load_profile(client).await;
                }
                Ok(false)
            }
            KeyCode::Char('s') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('s');
                } else {
                    app.node_state.toggle_completion_mode();
                }
                Ok(false)
            }
            KeyCode::Tab => {
                app.node_state.toggle_completion_mode();
                Ok(false)
            }
            KeyCode::Char('t') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('t');
                }
                Ok(false)
            }
            KeyCode::Char('o') => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('o');
                }
                Ok(false)
            }
            KeyCode::Backspace => {
                if app.node_state.is_completion_mode {
                    app.node_state.delete_char();
                }
                Ok(false)
            }
            KeyCode::Char(ch) => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char(ch);
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Aggregate view key mapping
pub struct AggregateKeyMap;

impl AggregateKeyMap {
    pub fn new() -> Self {
        Self
    }
}

impl KeyMap for AggregateKeyMap {
    async fn handle_key(
        &self,
        app: &mut App,
        key: KeyEvent,
        client: &V2exClient,
    ) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => Ok(true),
            KeyCode::Char('?') => {
                app.previous_view = Some(app.view);
                app.view = View::Help;
                Ok(false)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.view = View::TopicList;
                app.ui_state.error = None;
                Ok(false)
            }
            KeyCode::Char('n') => {
                app.aggregate_state.next_item();
                Ok(false)
            }
            KeyCode::Down => {
                app.aggregate_state.next_item();
                Ok(false)
            }
            KeyCode::Char('p') => {
                app.aggregate_state.previous_item();
                Ok(false)
            }
            KeyCode::Up => {
                app.aggregate_state.previous_item();
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(item) = app.aggregate_state.items.get(app.aggregate_state.selected) {
                    let topic_id = item.extract_topic_id();

                    if let Some(topic_id) = topic_id {
                        app.previous_view = Some(app.view);
                        app.view = View::TopicDetail;
                        app.topic_state.show_replies = true;
                        app.load_topic_detail(client, topic_id).await;
                        app.load_topic_replies(client, topic_id, false).await;
                        app.ui_state.status_message =
                            format!("Loading topic {} from RSS", topic_id);
                    } else {
                        app.ui_state.status_message =
                            "No topic ID found in RSS link, opening in browser instead".to_string();
                        app.open_selected_aggregate_in_browser();
                    }
                }
                Ok(false)
            }
            KeyCode::Char('r') => {
                app.open_selected_aggregate_in_browser();
                Ok(false)
            }
            KeyCode::Char('g') => {
                app.load_aggregate(client).await;
                Ok(false)
            }
            KeyCode::Char('a') => {
                // Switch to apple tab
                app.switch_aggregate_tab(client, "apple").await;
                Ok(false)
            }
            KeyCode::Char('m') => {
                app.previous_view = Some(app.view);
                app.view = View::Notifications;
                app.load_notifications(client).await;
                Ok(false)
            }
            KeyCode::Char('u') => {
                app.previous_view = Some(app.view);
                app.view = View::Profile;
                app.load_profile(client).await;
                Ok(false)
            }
            KeyCode::Char('s') => {
                app.previous_view = Some(app.view);
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                Ok(false)
            }
            KeyCode::Char('t') => {
                // Switch to tech tab
                app.switch_aggregate_tab(client, "tech").await;
                Ok(false)
            }
            KeyCode::Char('o') => {
                app.open_selected_aggregate_in_browser();
                Ok(false)
            }
            KeyCode::Char('<') => {
                app.aggregate_state.selected = 0;
                Ok(false)
            }
            KeyCode::Char('>') => {
                if !app.aggregate_state.items.is_empty() {
                    app.aggregate_state.selected = app.aggregate_state.items.len() - 1;
                }
                Ok(false)
            }
            KeyCode::Char(ch) => {
                // Handle tab switching
                let tab = match ch {
                    'c' => Some("creative"),
                    'k' => Some("play"),
                    'j' => Some("jobs"),
                    'd' => Some("deals"),
                    'y' => Some("city"),
                    'z' => Some("qna"),
                    'i' => Some("index"),
                    _ => None,
                };
                if let Some(tab) = tab {
                    app.switch_aggregate_tab(client, tab).await;
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

/// Main event handler that routes to appropriate key map
pub struct EventHandler<'a> {
    client: &'a V2exClient,
    link_map: LinkSelectionKeyMap,
    topic_list_map: TopicListKeyMap,
    topic_detail_map: TopicDetailKeyMap,
    notifications_map: NotificationsKeyMap,
    profile_map: ProfileKeyMap,
    help_map: HelpKeyMap,
    node_select_map: NodeSelectKeyMap,
    aggregate_map: AggregateKeyMap,
}

impl<'a> EventHandler<'a> {
    pub fn new(client: &'a V2exClient) -> Self {
        Self {
            client,
            link_map: LinkSelectionKeyMap::new(),
            topic_list_map: TopicListKeyMap::new(),
            topic_detail_map: TopicDetailKeyMap::new(),
            notifications_map: NotificationsKeyMap::new(),
            profile_map: ProfileKeyMap::new(),
            help_map: HelpKeyMap::new(),
            node_select_map: NodeSelectKeyMap::new(),
            aggregate_map: AggregateKeyMap::new(),
        }
    }

    pub async fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        // Check link selection mode first - it has highest priority
        if app.topic_state.link_input_state.is_active {
            return self.link_map.handle_key(app, key, self.client).await;
        }

        // Route to appropriate key map based on current view
        match app.view {
            View::TopicList => self.topic_list_map.handle_key(app, key, self.client).await,
            View::TopicDetail => self.topic_detail_map.handle_key(app, key, self.client).await,
            View::Notifications => self.notifications_map.handle_key(app, key, self.client).await,
            View::Profile => self.profile_map.handle_key(app, key, self.client).await,
            View::Help => self.help_map.handle_key(app, key, self.client).await,
            View::NodeSelect => self.node_select_map.handle_key(app, key, self.client).await,
            View::Aggregate => self.aggregate_map.handle_key(app, key, self.client).await,
            View::TokenInput => {
                // Token input is handled separately in main.rs
                Ok(false)
            }
        }
    }
}
