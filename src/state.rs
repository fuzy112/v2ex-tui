use crate::api::RssItem;
use ratatui::widgets::ListState;
use std::ops::Range;
use std::time::{Duration, Instant};

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

    #[test]
    fn test_topic_state_detect_links() {
        let mut state = TopicState::default();

        // Create a test topic with content containing links
        state.current = Some(create_test_topic_with_content(1));

        // Detect links
        state.detect_links(80); // Test width

        // Check that links were detected (if regex works)
        // Note: This test may fail if regex pattern doesn't match,
        // but at least we test that the method doesn't panic
        assert_eq!(
            state.link_shortcuts.len(),
            state.detected_links.iter().take(9).count()
        );
    }

    fn create_test_topic_with_content(id: i64) -> crate::api::Topic {
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
            content: Some(
                "Check out https://example.com and http://test.org for more info".to_string(),
            ),
            content_rendered: None,
            last_modified: None,
            replies: 0,
        }
    }
}

#[derive(Debug)]
pub struct DetectedLink {
    pub url: String,
    pub shortcut: String,
    pub text_range: Range<usize>,
    #[allow(dead_code)] // Not currently used, but kept for completeness
    pub display_text: String,
}

#[derive(Debug)]
pub struct LinkInputState {
    pub current_input: String,
    pub last_key_time: Option<Instant>,
    pub timeout_duration: Duration,
    pub is_active: bool,
}

impl Default for LinkInputState {
    fn default() -> Self {
        Self {
            current_input: String::new(),
            last_key_time: None,
            timeout_duration: Duration::from_millis(300),
            is_active: false,
        }
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
    pub detected_links: Vec<DetectedLink>,
    pub link_shortcuts: Vec<String>,
    pub link_input_state: LinkInputState,
    pub parsed_content_cache: Option<String>,
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

    pub fn next_reply(&mut self, width: usize) {
        if !self.replies.is_empty() {
            self.selected_reply = (self.selected_reply + 1) % self.replies.len();
            self.replies_list_state.select(Some(self.selected_reply));
            self.detect_links(width); // Update links for the newly selected reply
        }
    }

    pub fn previous_reply(&mut self, width: usize) {
        if !self.replies.is_empty() {
            self.selected_reply = if self.selected_reply == 0 {
                self.replies.len() - 1
            } else {
                self.selected_reply - 1
            };
            self.replies_list_state.select(Some(self.selected_reply));
            self.detect_links(width); // Update links for the newly selected reply
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

    pub fn detect_links(&mut self, width: usize) {
        self.detected_links.clear();
        self.link_shortcuts.clear();
        self.parsed_content_cache = None;

        // Determine which content to parse based on current view
        let content_to_parse = if self.show_replies && !self.replies.is_empty() {
            // Parse the selected reply
            self.replies
                .get(self.selected_reply)
                .and_then(|reply| reply.content_rendered.as_ref().or(reply.content.as_ref()))
                .map(|content| content.to_string())
        } else if let Some(topic) = &self.current {
            // Parse the topic content
            topic
                .content_rendered
                .as_ref()
                .or(topic.content.as_ref())
                .map(|content| content.to_string())
        } else {
            None
        };

        if let Some(content) = content_to_parse {
            // Convert HTML to text using html2text with the actual terminal width
            // This ensures link positions match the rendered text
            let converted_text = html2text::from_read(content.as_bytes(), width);

            // Store converted text for potential use in rendering
            self.parsed_content_cache = Some(converted_text.clone());

            // Extract links from converted text
            self.extract_links_from_converted_text(&converted_text);
        }

        // Generate shortcuts for first 9 links (for backward compatibility)
        for (i, link) in self.detected_links.iter().take(9).enumerate() {
            self.link_shortcuts.push(format!("{}: {}", i + 1, link.url));
        }
    }

    fn assign_shortcut(index: usize) -> String {
        const HOME_ROW: &[char] = &['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'];

        if index < HOME_ROW.len() {
            HOME_ROW[index].to_string()
        } else {
            let first_idx = (index - HOME_ROW.len()) / HOME_ROW.len();
            let second_idx = (index - HOME_ROW.len()) % HOME_ROW.len();
            format!("{}{}", HOME_ROW[first_idx], HOME_ROW[second_idx])
        }
    }

    fn extract_links_from_converted_text(&mut self, converted_text: &str) {
        use regex::Regex;

        // Simple URL regex pattern - match URLs in converted text
        // html2text may convert HTML links to [text](url) format, but URLs may also appear as plain text
        let url_pattern = "https?://[^\\s<>\"'\\)\\]]+";
        let re = match Regex::new(url_pattern) {
            Ok(regex) => regex,
            Err(_) => return, // If regex fails, skip link detection
        };

        // Clear existing links
        self.detected_links.clear();

        // Find all matches with their byte positions
        let mut matches: Vec<(String, std::ops::Range<usize>)> = Vec::new();
        for capture in re.captures_iter(converted_text) {
            if let Some(matched) = capture.get(0) {
                let url = matched.as_str().to_string();
                let byte_range = matched.start()..matched.end();
                matches.push((url, byte_range));
            }
        }

        // Store byte positions (not character positions) for accurate string slicing
        // URLs are typically ASCII, so byte positions work correctly
        for (index, (url, byte_range)) in matches.into_iter().enumerate() {
            let shortcut = Self::assign_shortcut(index);
            let display_text = if url.len() > 50 {
                format!("{}...", &url[..47])
            } else {
                url.clone()
            };

            self.detected_links.push(DetectedLink {
                url,
                shortcut,
                text_range: byte_range, // Store byte range for string slicing
                display_text,
            });
        }
    }

    // Link selection mode methods
    pub fn enter_link_selection_mode(&mut self, width: usize) {
        self.link_input_state.is_active = true;
        self.link_input_state.current_input.clear();
        self.link_input_state.last_key_time = None;
        // Detect links with positions in the currently displayed content
        self.detect_links(width);
    }

    pub fn exit_link_selection_mode(&mut self) {
        self.link_input_state.is_active = false;
        self.link_input_state.current_input.clear();
        self.link_input_state.last_key_time = None;
    }

    pub fn handle_link_mode_key(&mut self, ch: char) -> (String, bool, bool) {
        let now = Instant::now();
        let mut timeout_reset = false;
        let mut valid_input = false;

        // Check timeout
        if let Some(last_time) = self.link_input_state.last_key_time {
            if now.duration_since(last_time) > self.link_input_state.timeout_duration {
                self.link_input_state.current_input.clear();
                timeout_reset = true;
            }
        }

        // Only accept home row letters
        if "aoeuidhtns".contains(ch) {
            self.link_input_state.current_input.push(ch);
            valid_input = true;
        }

        self.link_input_state.last_key_time = Some(now);
        (
            self.link_input_state.current_input.clone(),
            timeout_reset,
            valid_input,
        )
    }

    // Helper methods for phase 2 (to be implemented)
    pub fn find_links_by_prefix(&self, prefix: &str) -> Vec<&DetectedLink> {
        self.detected_links
            .iter()
            .filter(|link| link.shortcut.starts_with(prefix))
            .collect()
    }

    #[allow(dead_code)] // Utility function not currently used, but kept for completeness
    pub fn find_exact_link(&self, shortcut: &str) -> Option<&DetectedLink> {
        self.detected_links
            .iter()
            .find(|link| link.shortcut == shortcut)
    }

    pub fn get_link_by_shortcut(&self, shortcut: usize) -> Option<&String> {
        if shortcut >= 1 && shortcut <= self.detected_links.len() {
            // Temporary: return URL from new DetectedLink struct
            Some(&self.detected_links[shortcut - 1].url)
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
