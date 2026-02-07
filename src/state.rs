use crate::api::RssItem;
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct AggregateState {
    pub items: Vec<RssItem>,
    pub selected: usize,
    pub current_tab: String,
}

impl AggregateState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            current_tab: "index".to_string(),
        }
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn previous_item(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn switch_tab(&mut self, tab: &str) {
        self.current_tab = tab.to_string();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_state_next_topic() {
        let mut state = TopicState::default();

        // Empty topics - should not panic
        state.next_topic();
        assert_eq!(state.selected, 0);

        // Add topics
        state.topics = vec![
            create_test_topic(1),
            create_test_topic(2),
            create_test_topic(3),
        ];

        // Next topic
        state.next_topic();
        assert_eq!(state.selected, 1);

        state.next_topic();
        assert_eq!(state.selected, 2);

        // Wrap around
        state.next_topic();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_topic_state_previous_topic() {
        let mut state = TopicState::default();

        // Empty topics - should not panic
        state.previous_topic();
        assert_eq!(state.selected, 0);

        // Add topics
        state.topics = vec![
            create_test_topic(1),
            create_test_topic(2),
            create_test_topic(3),
        ];
        state.selected = 1;

        // Previous topic
        state.previous_topic();
        assert_eq!(state.selected, 0);

        // Wrap around
        state.previous_topic();
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn test_topic_state_scroll() {
        let mut state = TopicState::default();

        // Scroll down
        state.scroll_down();
        assert_eq!(state.scroll, 3);

        state.scroll_down();
        assert_eq!(state.scroll, 6);

        // Scroll up
        state.scroll_up();
        assert_eq!(state.scroll, 3);

        state.scroll_up();
        assert_eq!(state.scroll, 0);

        // Should not go below 0
        state.scroll_up();
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn test_notification_state_next() {
        let mut state = NotificationState::default();

        // Empty - should not panic
        state.next();
        assert_eq!(state.selected, 0);

        // Add notifications
        state.notifications = vec![create_test_notification(1), create_test_notification(2)];

        state.next();
        assert_eq!(state.selected, 1);

        // Wrap around
        state.next();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_notification_state_previous() {
        let mut state = NotificationState::default();

        // Add notifications
        state.notifications = vec![create_test_notification(1), create_test_notification(2)];
        state.selected = 1;

        state.previous();
        assert_eq!(state.selected, 0);

        // Wrap around
        state.previous();
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_node_state_switch_node() {
        let mut state = NodeState::new();

        assert_eq!(state.current_node, "python");
        assert_eq!(state.page, 1);

        state.switch_node("rust");
        assert_eq!(state.current_node, "rust");
        assert_eq!(state.page, 1);
    }

    #[test]
    fn test_node_state_select_current_node() {
        let mut state = NodeState::new();

        // Select from favorite nodes
        state.selected = 1; // programmer
        let result = state.select_current_node();
        assert_eq!(result, Some("programmer".to_string()));
        assert_eq!(state.current_node, "programmer");
    }

    #[test]
    fn test_token_state_insert_delete() {
        let mut state = TokenState::default();

        // Insert characters
        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.input, "hi");
        assert_eq!(state.cursor, 2);

        // Move cursor left
        state.move_cursor_left();
        assert_eq!(state.cursor, 1);

        // Insert in middle: cursor at 1, inserts 'e' -> "hei"
        state.insert_char('e');
        assert_eq!(state.input, "hei");
        assert_eq!(state.cursor, 2);

        // Delete character: cursor at 2, deletes char at position 1 ('e')
        state.delete_char();
        assert_eq!(state.input, "hi");
        assert_eq!(state.cursor, 1);
    }

    // Helper functions
    fn create_test_topic(id: i64) -> crate::api::Topic {
        crate::api::Topic {
            id,
            node: None,
            member: None,
            last_reply_by: None,
            last_touched: None,
            title: format!("Test Topic {}", id),
            url: format!("https://v2ex.com/t/{}", id),
            created: 0,
            deleted: None,
            content: None,
            content_rendered: None,
            last_modified: None,
            replies: 0,
        }
    }

    fn create_test_notification(id: i64) -> crate::api::Notification {
        crate::api::Notification {
            id,
            member_id: 1,
            member: None,
            for_member_id: 1,
            text: format!("Test notification {}", id),
            payload: None,
            payload_rendered: None,
            created: 0,
        }
    }

    fn create_test_rss_item(id: usize) -> crate::api::RssItem {
        crate::api::RssItem {
            title: format!("Test RSS item {}", id),
            link: format!("https://example.com/{}", id),
            date: "2026-02-07 12:00".to_string(),
            author: Some("test".to_string()),
        }
    }

    #[test]
    fn test_aggregate_state_next_item() {
        let mut state = AggregateState::new();

        // Empty items - should not panic
        state.next_item();
        assert_eq!(state.selected, 0);

        // Add items
        state.items = vec![
            create_test_rss_item(1),
            create_test_rss_item(2),
            create_test_rss_item(3),
        ];

        // Next item
        state.next_item();
        assert_eq!(state.selected, 1);

        state.next_item();
        assert_eq!(state.selected, 2);

        // Wrap around
        state.next_item();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_aggregate_state_previous_item() {
        let mut state = AggregateState::new();

        // Empty items - should not panic
        state.previous_item();
        assert_eq!(state.selected, 0);

        // Add items
        state.items = vec![
            create_test_rss_item(1),
            create_test_rss_item(2),
            create_test_rss_item(3),
        ];
        state.selected = 1;

        // Previous item
        state.previous_item();
        assert_eq!(state.selected, 0);

        // Wrap around
        state.previous_item();
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn test_aggregate_state_switch_tab() {
        let mut state = AggregateState::new();
        assert_eq!(state.current_tab, "index");

        state.switch_tab("tech");
        assert_eq!(state.current_tab, "tech");
        assert_eq!(state.selected, 0);

        state.switch_tab("creative");
        assert_eq!(state.current_tab, "creative");
        assert_eq!(state.selected, 0);
    }
}

#[derive(Debug, Default)]
pub struct TopicState {
    pub topics: Vec<crate::api::Topic>,
    pub selected: usize,
    pub current: Option<crate::api::Topic>,
    pub replies: Vec<crate::api::Reply>,
    pub replies_page: i32,
    pub scroll: usize,
    pub selected_reply: usize,
    pub replies_list_state: ListState,
    pub show_replies: bool,
}

impl TopicState {
    pub fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected = (self.selected + 1) % self.topics.len();
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected = if self.selected == 0 {
                self.topics.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn next_reply(&mut self) {
        if !self.replies.is_empty() {
            self.selected_reply = (self.selected_reply + 1) % self.replies.len();
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    pub fn previous_reply(&mut self) {
        if !self.replies.is_empty() {
            self.selected_reply = if self.selected_reply == 0 {
                self.replies.len() - 1
            } else {
                self.selected_reply - 1
            };
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll >= 3 {
            self.scroll -= 3;
        } else {
            self.scroll = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll += 3;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
        self.selected_reply = 0;
        if self.replies.is_empty() {
            self.replies_list_state.select(None);
        } else {
            self.replies_list_state.select(Some(0));
        }
    }

    pub fn find_current_topic_index(&self) -> Option<usize> {
        if let Some(current_topic) = &self.current {
            self.topics
                .iter()
                .position(|topic| topic.id == current_topic.id)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct NotificationState {
    pub notifications: Vec<crate::api::Notification>,
    pub selected: usize,
}

impl NotificationState {
    pub fn next(&mut self) {
        if !self.notifications.is_empty() {
            self.selected = (self.selected + 1) % self.notifications.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.notifications.is_empty() {
            self.selected = if self.selected == 0 {
                self.notifications.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

#[derive(Debug, Default)]
pub struct NodeState {
    pub favorite_nodes: Vec<(String, String)>,
    pub all_nodes: Vec<(String, String)>,
    pub original_favorite_nodes: Vec<(String, String)>,
    pub selected: usize,
    pub current_node: String,
    pub page: i32,
    pub completion_input: String,
    pub completion_cursor: usize,
    pub is_completion_mode: bool,
}

impl NodeState {
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

        let all_nodes = crate::nodes::get_all_nodes().to_vec();

        Self {
            favorite_nodes: favorite_nodes.clone(),
            all_nodes,
            original_favorite_nodes: favorite_nodes,
            selected: 0,
            current_node: "python".to_string(),
            page: 1,
            completion_input: String::new(),
            completion_cursor: 0,
            is_completion_mode: false,
        }
    }

    pub fn next_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected = (self.selected + 1) % self.favorite_nodes.len();
        }
    }

    pub fn previous_node(&mut self) {
        if !self.favorite_nodes.is_empty() {
            self.selected = if self.selected == 0 {
                self.favorite_nodes.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn select_current_node(&mut self) -> Option<String> {
        if let Some((node_name, _)) = self.favorite_nodes.get(self.selected) {
            self.current_node = node_name.clone();
            self.page = 1;
            Some(node_name.clone())
        } else if self.is_completion_mode {
            let node_name = self.completion_input.trim();
            if !node_name.is_empty() {
                self.current_node = node_name.to_string();
                self.page = 1;
                Some(node_name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn switch_node(&mut self, node: &str) {
        self.current_node = node.to_string();
        self.page = 1;
    }

    pub fn insert_char(&mut self, ch: char) {
        let byte_pos = self
            .completion_input
            .char_indices()
            .nth(self.completion_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.completion_input.len());
        self.completion_input.insert(byte_pos, ch);
        self.completion_cursor += 1;
        if self.is_completion_mode {
            self.update_suggestions();
        }
    }

    pub fn delete_char(&mut self) {
        if self.completion_cursor > 0 {
            let byte_pos = self
                .completion_input
                .char_indices()
                .nth(self.completion_cursor - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next_byte_pos = self
                .completion_input
                .char_indices()
                .nth(self.completion_cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.completion_input.len());
            self.completion_input.drain(byte_pos..next_byte_pos);
            self.completion_cursor -= 1;
            if self.is_completion_mode {
                self.update_suggestions();
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.completion_cursor > 0 {
            self.completion_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.completion_cursor < self.completion_input.chars().count() {
            self.completion_cursor += 1;
        }
    }

    pub fn toggle_completion_mode(&mut self) {
        self.is_completion_mode = !self.is_completion_mode;
        if self.is_completion_mode {
            self.update_suggestions();
        } else {
            self.favorite_nodes = self.original_favorite_nodes.clone();
            self.selected = 0;
        }
    }

    pub fn update_suggestions(&mut self) {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let input = self.completion_input.trim();
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
        self.selected = 0;
    }

    pub fn reset_selection(&mut self) {
        self.completion_input.clear();
        self.completion_cursor = 0;
        self.is_completion_mode = false;
    }
}

#[derive(Debug, Default)]
pub struct TokenState {
    pub input: String,
    pub cursor: usize,
}

impl TokenState {
    pub fn insert_char(&mut self, ch: char) {
        let byte_pos = self
            .input
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len());
        self.input.insert(byte_pos, ch);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            let byte_pos = self
                .input
                .char_indices()
                .nth(self.cursor - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next_byte_pos = self
                .input
                .char_indices()
                .nth(self.cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.input.len());
            self.input.drain(byte_pos..next_byte_pos);
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.input.chars().count() {
            self.cursor += 1;
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        use anyhow::Context;
        let config_dir = crate::api::V2exClient::config_dir()?;
        let token_path = config_dir.join("token.txt");
        std::fs::write(&token_path, self.input.trim())
            .with_context(|| format!("Failed to write token to {:?}", token_path))?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub loading: bool,
    pub error: Option<String>,
    pub status_message: String,
    pub theme: crate::ui::Theme,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            loading: false,
            error: None,
            status_message: "Press '?' for help".to_string(),
            theme: crate::ui::Theme::default(),
        }
    }
}
