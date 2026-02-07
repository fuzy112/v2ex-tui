use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::api::V2exClient;
use crate::app::{App, View};

pub struct EventHandler<'a> {
    client: &'a V2exClient,
}

impl<'a> EventHandler<'a> {
    pub fn new(client: &'a V2exClient) -> Self {
        Self { client }
    }

    pub async fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => self.handle_q(app, key),
            KeyCode::Esc => self.handle_esc(app),
            KeyCode::Char('?') => self.handle_help(app, key),
            KeyCode::Char('h') | KeyCode::Left => self.handle_back(app, key),
            KeyCode::Char('n') => self.handle_n(app, key).await,
            KeyCode::Down => self.handle_down(app),
            KeyCode::Char('p') => self.handle_p(app, key).await,
            KeyCode::Up => self.handle_up(app),
            KeyCode::Char('l') | KeyCode::Right => self.handle_forward(app, key).await,
            KeyCode::Enter => self.handle_enter(app).await,
            KeyCode::Char('r') => self.handle_r(app, key),
            KeyCode::Char('g') => self.handle_g(app, key).await,
            KeyCode::Char('a') => self.handle_a(app, key),
            KeyCode::Char('m') => self.handle_m(app, key).await,
            KeyCode::Char('u') => self.handle_u(app, key).await,
            KeyCode::Char('s') => self.handle_s(app),
            KeyCode::Tab => self.handle_tab(app),
            KeyCode::Char('t') => self.handle_t(app, key).await,
            KeyCode::Char('o') => self.handle_o(app, key),
            KeyCode::Char('N') => self.handle_capital_n(app, key).await,
            KeyCode::Char('P') => self.handle_capital_p(app, key).await,
            KeyCode::Char('1') => self.handle_number(app, key, "python").await,
            KeyCode::Char('2') => self.handle_number(app, key, "programmer").await,
            KeyCode::Char('3') => self.handle_number(app, key, "share").await,
            KeyCode::Char('4') => self.handle_number(app, key, "create").await,
            KeyCode::Char('5') => self.handle_number(app, key, "jobs").await,
            KeyCode::Char('6') => self.handle_number(app, key, "go").await,
            KeyCode::Char('7') => self.handle_number(app, key, "rust").await,
            KeyCode::Char('8') => self.handle_number(app, key, "javascript").await,
            KeyCode::Char('9') => self.handle_number(app, key, "linux").await,
            KeyCode::PageDown => self.handle_page_down(app).await,
            KeyCode::PageUp => self.handle_page_up(app).await,
            KeyCode::Char('+') => self.handle_plus(app, key).await,
            KeyCode::Char('<') => self.handle_less_than(app, key),
            KeyCode::Char('>') => self.handle_greater_than(app, key),
            KeyCode::Char(ch) => self.handle_char(app, ch),
            KeyCode::Backspace => self.handle_backspace(app),
            _ => Ok(false),
        }
    }

    fn handle_q(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('q');
            Ok(false)
        } else {
            match app.view {
                View::TopicList => Ok(true),
                View::NodeSelect => {
                    app.view = View::TopicList;
                    Ok(false)
                }
                _ => {
                    app.view = View::TopicList;
                    app.ui_state.error = None;
                    Ok(false)
                }
            }
        }
    }

    fn handle_esc(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => Ok(true),
            View::NodeSelect => {
                app.view = View::TopicList;
                Ok(false)
            }
            _ => {
                app.view = View::TopicList;
                app.ui_state.error = None;
                Ok(false)
            }
        }
    }

    fn handle_help(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('?');
        } else {
            app.view = View::Help;
        }
        Ok(false)
    }

    fn handle_back(&self, app: &mut App, key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            if key.code == KeyCode::Char('h') {
                app.node_state.insert_char('h');
            } else {
                app.node_state.move_cursor_left();
            }
        } else {
            match app.view {
                View::NodeSelect => {
                    app.view = View::TopicList;
                }
                _ => {
                    if app.view != View::TopicList {
                        app.view = View::TopicList;
                        app.ui_state.error = None;
                    }
                }
            }
        }
        Ok(false)
    }

    async fn handle_n(&self, app: &mut App, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if app.view == View::NodeSelect {
                app.node_state.next_node();
            }
        } else if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('n');
        } else {
            match app.view {
                View::TopicList => app.topic_state.next_topic(),
                View::Notifications => app.notification_state.next(),
                View::NodeSelect => app.node_state.next_node(),
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.topic_state.next_reply();
                    } else {
                        app.topic_state.scroll_down();
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_down(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => app.topic_state.next_topic(),
            View::Notifications => app.notification_state.next(),
            View::NodeSelect => {
                if !app.node_state.is_completion_mode {
                    app.node_state.next_node();
                }
            }
            View::TopicDetail => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.next_reply();
                } else {
                    app.topic_state.scroll_down();
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_p(&self, app: &mut App, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if app.view == View::NodeSelect {
                app.node_state.previous_node();
            }
        } else if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('p');
        } else {
            match app.view {
                View::TopicList => app.topic_state.previous_topic(),
                View::Notifications => app.notification_state.previous(),
                View::NodeSelect => app.node_state.previous_node(),
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.topic_state.previous_reply();
                    } else {
                        app.topic_state.scroll_up();
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_up(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => app.topic_state.previous_topic(),
            View::Notifications => app.notification_state.previous(),
            View::NodeSelect => {
                if !app.node_state.is_completion_mode {
                    app.node_state.previous_node();
                }
            }
            View::TopicDetail => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.previous_reply();
                } else {
                    app.topic_state.scroll_up();
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_forward(&self, app: &mut App, key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            if key.code == KeyCode::Char('l') {
                app.node_state.insert_char('l');
            } else {
                app.node_state.move_cursor_right();
            }
        } else {
            match app.view {
                View::TopicList => {
                    if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                        let topic_id = topic.id;
                        app.view = View::TopicDetail;
                        app.topic_state.show_replies = true;
                        app.load_topic_detail(self.client, topic_id).await;
                        app.load_topic_replies(self.client, topic_id, false).await;
                    }
                }
                View::Notifications => {
                    if let Some(notification) = app
                        .notification_state
                        .notifications
                        .get(app.notification_state.selected)
                    {
                        let topic_id = notification.extract_topic_id();
                        let reply_id = notification.extract_reply_id();

                        if let Some(topic_id) = topic_id {
                            app.view = View::TopicDetail;
                            app.topic_state.show_replies = true;
                            app.load_topic_detail(self.client, topic_id).await;
                            app.load_topic_replies(self.client, topic_id, false).await;

                            if let Some(reply_id) = reply_id {
                                app.ui_state.status_message =
                                    format!("Jumping to topic {} (reply #{})", topic_id, reply_id);
                            } else {
                                app.ui_state.status_message =
                                    format!("Jumping to topic {}", topic_id);
                            }
                        } else {
                            app.ui_state.status_message =
                                "No topic link found in this notification".to_string();
                        }
                    }
                }
                View::NodeSelect => {
                    app.node_state.select_current_node();
                    app.node_state.reset_selection();
                    app.view = View::TopicList;
                    app.load_topics(self.client, false).await;
                }
                _ => {}
            }
        }
        Ok(false)
    }

    async fn handle_enter(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => {
                if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                    let topic_id = topic.id;
                    app.view = View::TopicDetail;
                    app.topic_state.show_replies = true;
                    app.load_topic_detail(self.client, topic_id).await;
                    app.load_topic_replies(self.client, topic_id, false).await;
                }
            }
            View::Notifications => {
                if let Some(notification) = app
                    .notification_state
                    .notifications
                    .get(app.notification_state.selected)
                {
                    let topic_id = notification.extract_topic_id();
                    let reply_id = notification.extract_reply_id();

                    if let Some(topic_id) = topic_id {
                        app.view = View::TopicDetail;
                        app.topic_state.show_replies = true;
                        app.load_topic_detail(self.client, topic_id).await;
                        app.load_topic_replies(self.client, topic_id, false).await;

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
            }
            View::NodeSelect => {
                app.node_state.select_current_node();
                app.node_state.reset_selection();
                app.view = View::TopicList;
                app.load_topics(self.client, false).await;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_r(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('r');
        } else {
            match app.view {
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.open_selected_reply_in_browser();
                    } else {
                        app.open_current_topic_in_browser();
                    }
                }
                View::TopicList => app.open_selected_topic_in_browser(),
                View::Notifications => app.open_notification_in_browser(),
                _ => {}
            }
        }
        Ok(false)
    }

    async fn handle_g(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('g');
        } else {
            match app.view {
                View::TopicList => app.load_topics(self.client, false).await,
                View::TopicDetail => {
                    if let Some(ref topic) = app.topic_state.current {
                        let topic_id = topic.id;
                        app.load_topic_detail(self.client, topic_id).await;
                        app.load_topic_replies(self.client, topic_id, false).await;
                    }
                }
                View::Notifications => app.load_notifications(self.client).await,
                View::Profile => app.load_profile(self.client).await,
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_a(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('a');
        }
        Ok(false)
    }

    async fn handle_m(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('m');
        } else {
            app.view = View::Notifications;
            app.load_notifications(self.client).await;
        }
        Ok(false)
    }

    async fn handle_u(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('u');
        } else {
            app.view = View::Profile;
            app.load_profile(self.client).await;
        }
        Ok(false)
    }

    fn handle_s(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::NodeSelect => {
                if app.node_state.is_completion_mode {
                    app.node_state.insert_char('s');
                } else {
                    app.node_state.toggle_completion_mode();
                }
            }
            _ => {
                app.view = View::NodeSelect;
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
            }
        }
        Ok(false)
    }

    fn handle_tab(&self, app: &mut App) -> Result<bool> {
        if app.view == View::NodeSelect {
            app.node_state.toggle_completion_mode();
        }
        Ok(false)
    }

    async fn handle_t(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('t');
        } else {
            match app.view {
                View::TopicList => {
                    if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                        let topic_id = topic.id;
                        app.view = View::TopicDetail;
                        app.topic_state.show_replies = true;
                        app.load_topic_detail(self.client, topic_id).await;
                        app.load_topic_replies(self.client, topic_id, false).await;
                    }
                }
                View::TopicDetail => {
                    app.topic_state.show_replies = !app.topic_state.show_replies;
                    app.topic_state.reset_scroll();
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_o(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('o');
        } else {
            match app.view {
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.open_selected_reply_in_browser();
                    } else {
                        app.open_current_topic_in_browser();
                    }
                }
                View::TopicList => app.open_selected_topic_in_browser(),
                View::Notifications => app.open_notification_in_browser(),
                _ => {}
            }
        }
        Ok(false)
    }

    async fn handle_capital_n(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('N');
        } else if app.view == View::TopicDetail {
            app.switch_to_next_topic(self.client).await;
        }
        Ok(false)
    }

    async fn handle_capital_p(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('P');
        } else if app.view == View::TopicDetail {
            app.switch_to_previous_topic(self.client).await;
        }
        Ok(false)
    }

    async fn handle_number(&self, app: &mut App, key: KeyEvent, node: &str) -> Result<bool> {
        let digit = match key.code {
            KeyCode::Char('1') => '1',
            KeyCode::Char('2') => '2',
            KeyCode::Char('3') => '3',
            KeyCode::Char('4') => '4',
            KeyCode::Char('5') => '5',
            KeyCode::Char('6') => '6',
            KeyCode::Char('7') => '7',
            KeyCode::Char('8') => '8',
            KeyCode::Char('9') => '9',
            _ => return Ok(false),
        };

        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char(digit);
        } else {
            app.node_state.switch_node(node);
            app.load_topics(self.client, false).await;
        }
        Ok(false)
    }

    async fn handle_page_down(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => {
                app.node_state.page += 1;
                app.load_topics(self.client, true).await;
            }
            View::TopicDetail => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    app.topic_state.selected_reply =
                        (app.topic_state.selected_reply + 5).min(app.topic_state.replies.len() - 1);
                } else {
                    app.topic_state.scroll += 15;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_page_up(&self, app: &mut App) -> Result<bool> {
        match app.view {
            View::TopicList => {
                if app.node_state.page > 1 {
                    app.node_state.page -= 1;
                    app.load_topics(self.client, false).await;
                }
            }
            View::TopicDetail => {
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
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_plus(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('+');
        } else if app.view == View::TopicList {
            app.node_state.page += 1;
            app.load_topics(self.client, true).await;
        } else if app.view == View::TopicDetail && app.topic_state.show_replies {
            if let Some(ref topic) = app.topic_state.current {
                app.load_topic_replies(self.client, topic.id, true).await;
            }
        }
        Ok(false)
    }

    fn handle_less_than(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('<');
        } else {
            match app.view {
                View::TopicList => app.topic_state.selected = 0,
                View::Notifications => app.notification_state.selected = 0,
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.topic_state.selected_reply = 0;
                        app.topic_state.replies_list_state.select(Some(0));
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_greater_than(&self, app: &mut App, _key: KeyEvent) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char('>');
        } else {
            match app.view {
                View::TopicList => {
                    if !app.topic_state.topics.is_empty() {
                        app.topic_state.selected = app.topic_state.topics.len() - 1;
                    }
                }
                View::Notifications => {
                    if !app.notification_state.notifications.is_empty() {
                        app.notification_state.selected =
                            app.notification_state.notifications.len() - 1;
                    }
                }
                View::TopicDetail => {
                    if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                        app.topic_state.selected_reply = app.topic_state.replies.len() - 1;
                        app.topic_state
                            .replies_list_state
                            .select(Some(app.topic_state.selected_reply));
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn handle_char(&self, app: &mut App, ch: char) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.insert_char(ch);
        }
        Ok(false)
    }

    fn handle_backspace(&self, app: &mut App) -> Result<bool> {
        if app.view == View::NodeSelect && app.node_state.is_completion_mode {
            app.node_state.delete_char();
        }
        Ok(false)
    }
}
