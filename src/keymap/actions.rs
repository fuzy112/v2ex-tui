use std::collections::HashMap;

use anyhow::Result;

use crate::api::V2exClient;
use crate::app::{App, View};

/// All possible actions that can be bound to keys
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    // Global actions
    QuitImmediate,
    RemoveFromHistory,
    HistoryBack,
    HistoryForward,
    ShowHelp,
    Refresh,
    ReloadConfig,

    // Navigation
    NavigateTo(View),
    NextItem,
    PreviousItem,
    FirstItem,
    LastItem,
    PageUp,
    PageDown,

    // View-specific
    LoadMoreTopics,
    LoadMoreReplies,
    ToggleReplies,
    CopyToClipboard,
    EnterLinkMode,
    ExitLinkMode,
    OpenInBrowser,
    SelectNode,
    SwitchNode(String),
    LinkSelect(String),

    // Topic actions
    OpenTopic,
    NextTopic,
    PreviousTopic,
    NextReply,
    PreviousReply,

    // Aggregate actions
    SwitchAggregateTab(String),
    OpenAggregateItem,
    RefreshAggregate,

    // Custom action with name
    Custom(String),
}

/// Registry mapping action names to Action variants
pub struct ActionRegistry {
    actions: HashMap<String, Action>,
}

impl ActionRegistry {
    /// Create a new action registry with all built-in actions
    pub fn new() -> Self {
        let mut registry = Self {
            actions: HashMap::new(),
        };
        registry.register_builtin_actions();
        registry
    }

    /// Register an action
    pub fn register(&mut self, name: impl Into<String>, action: Action) {
        self.actions.insert(name.into(), action);
    }

    /// Get an action by name
    pub fn get(&self, name: &str) -> Option<&Action> {
        self.actions.get(name)
    }

    /// Check if an action exists
    pub fn has_action(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }

    /// Execute an action
    pub async fn execute(
        &self,
        action: &Action,
        app: &mut App,
        client: &V2exClient,
    ) -> Result<bool> {
        use Action::*;

        match action {
            QuitImmediate => Ok(true),

            RemoveFromHistory => {
                if app.remove_current_from_history().is_none() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            HistoryBack => {
                app.history_back();
                Ok(false)
            }

            HistoryForward => {
                app.history_forward();
                Ok(false)
            }

            ShowHelp => {
                app.navigate_to(View::Help);
                Ok(false)
            }

            Refresh => {
                match app.view {
                    View::TopicList => app.load_topics(client, false).await,
                    View::TopicDetail => {
                        if let Some(ref topic) = app.topic_state.current {
                            let topic_id = topic.id;
                            app.load_topic_detail(client, topic_id).await;
                            app.load_topic_replies(client, topic_id, false).await;
                        }
                    }
                    View::Notifications => app.load_notifications(client).await,
                    View::Profile => app.load_profile(client).await,
                    View::Aggregate => app.load_aggregate(client).await,
                    _ => {}
                }
                Ok(false)
            }

            NextItem | NextTopic => {
                let at_last = app.topic_state.selected + 1 >= app.topic_state.topics.len();
                if at_last && !app.topic_state.topics.is_empty() {
                    let prev_page = app.node_state.page;
                    app.node_state.page += 1;
                    let prev_len = app.topic_state.topics.len();
                    app.load_topics(client, true).await;
                    if app.topic_state.topics.len() > prev_len {
                        app.topic_state.selected = prev_len;
                    } else {
                        app.node_state.page = prev_page;
                        app.ui_state.error = None;
                        app.ui_state.status_message = "Already at the last topic".to_string();
                    }
                } else {
                    app.topic_state.next_topic();
                }
                Ok(false)
            }

            PreviousItem | PreviousTopic => {
                if app.topic_state.selected == 0 {
                    app.ui_state.status_message = "Already at the first topic".to_string();
                } else {
                    app.topic_state.previous_topic();
                }
                Ok(false)
            }

            OpenTopic => {
                if let Some(topic) = app.topic_state.topics.get(app.topic_state.selected) {
                    let topic_id = topic.id;
                    app.topic_state.show_replies = false;
                    app.load_topic_detail(client, topic_id).await;
                    app.load_topic_replies(client, topic_id, false).await;
                    app.navigate_to(View::TopicDetail);
                }
                Ok(false)
            }

            LoadMoreTopics => {
                app.node_state.page += 1;
                app.load_topics(client, true).await;
                Ok(false)
            }

            FirstItem => {
                match app.view {
                    View::TopicList | View::TopicDetail => {
                        app.topic_state.selected = 0;
                    }
                    View::Aggregate => {
                        app.aggregate_state.selected = 0;
                    }
                    _ => {}
                }
                Ok(false)
            }

            LastItem => {
                match app.view {
                    View::TopicList => {
                        if !app.topic_state.topics.is_empty() {
                            app.topic_state.selected = app.topic_state.topics.len() - 1;
                        }
                    }
                    View::TopicDetail => {
                        if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                            app.topic_state.selected_reply = app.topic_state.replies.len() - 1;
                        }
                    }
                    View::Aggregate => {
                        if !app.aggregate_state.items.is_empty() {
                            app.aggregate_state.selected = app.aggregate_state.items.len() - 1;
                        }
                    }
                    _ => {}
                }
                Ok(false)
            }

            ToggleReplies => {
                app.topic_state.show_replies = !app.topic_state.show_replies;
                app.topic_state.reset_scroll();
                Ok(false)
            }

            NextReply => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    let at_last =
                        app.topic_state.selected_reply + 1 >= app.topic_state.replies.len();
                    let has_more = app.topic_state.replies.len()
                        < app
                            .topic_state
                            .current
                            .as_ref()
                            .map(|t| t.replies as usize)
                            .unwrap_or(0);

                    if at_last && has_more {
                        if let Some(ref topic) = app.topic_state.current {
                            let prev_len = app.topic_state.replies.len();
                            app.load_topic_replies(client, topic.id, true).await;
                            if app.topic_state.replies.len() > prev_len {
                                app.topic_state.selected_reply = prev_len;
                                app.topic_state.replies_list_state.select(Some(prev_len));
                                app.topic_state.detect_links(app.terminal_width);
                            } else {
                                app.ui_state.status_message =
                                    "Already at the last reply".to_string();
                            }
                        }
                    } else if at_last && !has_more {
                        app.ui_state.status_message = "Already at the last reply".to_string();
                    } else {
                        app.topic_state.next_reply(app.terminal_width);
                    }
                } else {
                    app.topic_state.scroll_down();
                }
                Ok(false)
            }

            PreviousReply => {
                if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                    if app.topic_state.selected_reply == 0 {
                        app.ui_state.status_message = "Already at the first reply".to_string();
                    } else {
                        app.topic_state.previous_reply(app.terminal_width);
                    }
                } else {
                    app.topic_state.scroll_up();
                }
                Ok(false)
            }

            LoadMoreReplies => {
                if app.topic_state.show_replies {
                    if let Some(ref topic) = app.topic_state.current {
                        let loaded_replies = app.topic_state.replies.len();
                        let total_replies = topic.replies as usize;
                        if loaded_replies < total_replies {
                            app.load_topic_replies(client, topic.id, true).await;
                        } else {
                            app.ui_state.status_message = "No more replies to load".to_string();
                        }
                    }
                }
                Ok(false)
            }

            EnterLinkMode => {
                app.topic_state
                    .enter_link_selection_mode(app.terminal_width);
                app.ui_state.status_message =
                    "Link mode: press a/o/e/u/i/d/h/t/n/s (home row), Esc/Ctrl+g to cancel"
                        .to_string();
                Ok(false)
            }

            ExitLinkMode => {
                app.topic_state.exit_link_selection_mode();
                app.ui_state.status_message = "Link selection cancelled".to_string();
                Ok(false)
            }

            OpenInBrowser => {
                match app.view {
                    View::TopicList => app.open_selected_topic_in_browser(),
                    View::TopicDetail => {
                        if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                            app.open_selected_reply_in_browser();
                        } else {
                            app.open_current_topic_in_browser();
                        }
                    }
                    View::Notifications => app.open_notification_in_browser(),
                    View::Aggregate => app.open_selected_aggregate_in_browser(),
                    _ => {}
                }
                Ok(false)
            }

            NavigateTo(View::Aggregate) => {
                app.load_aggregate(client).await;
                app.navigate_to(View::Aggregate);
                Ok(false)
            }

            NavigateTo(View::Notifications) => {
                app.load_notifications(client).await;
                app.navigate_to(View::Notifications);
                Ok(false)
            }

            NavigateTo(View::Profile) => {
                app.load_profile(client).await;
                app.navigate_to(View::Profile);
                Ok(false)
            }

            NavigateTo(View::NodeSelect) | SelectNode => {
                app.node_state.completion_input.clear();
                app.node_state.completion_cursor = 0;
                app.node_state.is_completion_mode = true;
                app.node_state.update_suggestions();
                app.navigate_to(View::NodeSelect);
                Ok(false)
            }

            NavigateTo(view) => {
                app.navigate_to(*view);
                Ok(false)
            }

            SwitchNode(node) => {
                app.node_state.switch_node(node);
                app.load_topics(client, false).await;
                Ok(false)
            }

            LinkSelect(shortcut) => {
                // Handle link selection by shortcut
                // Parse shortcut letter to index (a=1, o=2, e=3, etc.)
                let shortcut_chars = ['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'];
                let index = shortcut_chars
                    .iter()
                    .position(|&c| c == shortcut.chars().next().unwrap_or('a'))
                    .map(|i| i + 1)
                    .unwrap_or(1);

                if let Some(url) = app.topic_state.get_link_by_shortcut(index) {
                    let url = url.clone();
                    match crate::browser::Browser::open_url(&url) {
                        Ok(result) => {
                            app.ui_state.status_message = format!("Opening link: {}", result);
                        }
                        Err(e) => {
                            app.ui_state.error = Some(format!("Failed to open link: {}", e));
                        }
                    }
                    app.topic_state.exit_link_selection_mode();
                }
                Ok(false)
            }

            PageUp => {
                match app.view {
                    View::TopicList => {
                        if app.topic_state.selected >= 5 {
                            app.topic_state.selected -= 5;
                        } else {
                            app.topic_state.selected = 0;
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
                    View::Notifications => {
                        if app.notification_state.selected >= 5 {
                            app.notification_state.selected -= 5;
                        } else {
                            app.notification_state.selected = 0;
                        }
                    }
                    View::Aggregate => {
                        if app.aggregate_state.selected >= 5 {
                            app.aggregate_state.selected -= 5;
                        } else {
                            app.aggregate_state.selected = 0;
                        }
                    }
                    _ => {}
                }
                Ok(false)
            }

            PageDown => {
                match app.view {
                    View::TopicList => {
                        let len = app.topic_state.topics.len();
                        if len > 0 {
                            app.topic_state.selected = (app.topic_state.selected + 5).min(len - 1);
                        }
                    }
                    View::TopicDetail => {
                        if app.topic_state.show_replies && !app.topic_state.replies.is_empty() {
                            app.topic_state.selected_reply = (app.topic_state.selected_reply + 5)
                                .min(app.topic_state.replies.len() - 1);
                        } else {
                            app.topic_state.scroll += 15;
                        }
                    }
                    View::Notifications => {
                        let len = app.notification_state.notifications.len();
                        if len > 0 {
                            app.notification_state.selected =
                                (app.notification_state.selected + 5).min(len - 1);
                        }
                    }
                    View::Aggregate => {
                        let len = app.aggregate_state.items.len();
                        if len > 0 {
                            app.aggregate_state.selected =
                                (app.aggregate_state.selected + 5).min(len - 1);
                        }
                    }
                    _ => {}
                }
                Ok(false)
            }

            // Aggregate actions
            SwitchAggregateTab(tab) => {
                app.switch_aggregate_tab(client, tab).await;
                Ok(false)
            }

            OpenAggregateItem => {
                app.open_selected_aggregate_in_browser();
                Ok(false)
            }

            RefreshAggregate => {
                app.load_aggregate(client).await;
                Ok(false)
            }

            ReloadConfig => {
                // TODO: Implement config reload
                app.ui_state.status_message = "Config reload not yet implemented".to_string();
                Ok(false)
            }

            Custom(name) => {
                // Handle special custom action formats
                if name.starts_with("switch-tab:") {
                    let tab = &name[11..]; // Strip "switch-tab:" prefix
                    app.switch_aggregate_tab(client, tab).await;
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("Custom action '{}' not implemented", name))
                }
            }

            _ => Err(anyhow::anyhow!("Action not implemented")),
        }
    }

    /// Register all built-in actions
    fn register_builtin_actions(&mut self) {
        use Action::*;

        // Global actions
        self.register("quit-immediate", QuitImmediate);
        self.register("remove-from-history", RemoveFromHistory);
        self.register("history-back", HistoryBack);
        self.register("history-forward", HistoryForward);
        self.register("show-help", ShowHelp);
        self.register("refresh", Refresh);
        self.register("reload-config", ReloadConfig);

        // Navigation
        self.register("next-item", NextItem);
        self.register("previous-item", PreviousItem);
        self.register("first-item", FirstItem);
        self.register("last-item", LastItem);
        self.register("page-up", PageUp);
        self.register("page-down", PageDown);

        // View navigation
        self.register("go-to-aggregate", NavigateTo(View::Aggregate));
        self.register("go-to-notifications", NavigateTo(View::Notifications));
        self.register("go-to-profile", NavigateTo(View::Profile));
        self.register("go-to-node-select", NavigateTo(View::NodeSelect));
        self.register("go-to-topic-list", NavigateTo(View::TopicList));

        // Topic list
        self.register("next-topic", NextTopic);
        self.register("previous-topic", PreviousTopic);
        self.register("open-topic", OpenTopic);
        self.register("load-more-topics", LoadMoreTopics);

        // Topic detail
        self.register("toggle-replies", ToggleReplies);
        self.register("next-reply", NextReply);
        self.register("previous-reply", PreviousReply);
        self.register("load-more-replies", LoadMoreReplies);
        self.register("enter-link-mode", EnterLinkMode);
        self.register("exit-link-mode", ExitLinkMode);
        self.register("open-in-browser", OpenInBrowser);
        self.register("select-node", SelectNode);

        // Aggregate view
        self.register("open-aggregate-item", OpenAggregateItem);
        self.register("refresh-aggregate", RefreshAggregate);
        // Note: SwitchAggregateTab is registered dynamically via Custom action

        // Aggregate tab switches (individual actions for each tab)
        self.register("switch-tab-tech", Custom("switch-tab:tech".to_string()));
        self.register(
            "switch-tab-creative",
            Custom("switch-tab:creative".to_string()),
        );
        self.register("switch-tab-play", Custom("switch-tab:play".to_string()));
        self.register("switch-tab-apple", Custom("switch-tab:apple".to_string()));
        self.register("switch-tab-jobs", Custom("switch-tab:jobs".to_string()));
        self.register("switch-tab-deals", Custom("switch-tab:deals".to_string()));
        self.register("switch-tab-city", Custom("switch-tab:city".to_string()));
        self.register("switch-tab-qna", Custom("switch-tab:qna".to_string()));
        self.register("switch-tab-index", Custom("switch-tab:index".to_string()));
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_registry() {
        let registry = ActionRegistry::new();
        assert!(registry.has_action("quit-immediate"));
        assert!(registry.has_action("next-topic"));
        assert!(!registry.has_action("nonexistent"));
    }
}
