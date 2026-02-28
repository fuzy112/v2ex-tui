//! Configuration system
//!
//! This module handles loading and managing user configuration from Lisp files.

pub mod engine;
pub mod loader;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::BaseDirs;

use crate::app::View;
use crate::keymap::actions::ActionRegistry;
use crate::keymap::{KeyMap, KeyMapChain};

/// All configuration settings
#[derive(Clone, Debug)]
pub struct Config {
    // General settings
    pub initial_view: View,
    pub initial_tab: String,  // for aggregate view (e.g., "tech", "creative")
    pub initial_node: String, // for topic-list view (e.g., "python", "programmer")
    pub topics_per_page: usize,
    pub replies_per_page: usize,
    pub auto_refresh_interval: u64, // seconds, 0 = disabled
    pub key_sequence_timeout: u64,  // milliseconds

    // Node settings
    pub favorite_nodes: Vec<(String, String)>, // (name, display_name)
    #[allow(dead_code)] // Reserved for future feature: Alternative node key mapping system
    pub quick_node_keys: HashMap<char, String>, // '1' -> "python"

    // Browser settings
    pub browser_command: Option<Vec<String>>,

    // Theme settings
    pub theme: ThemePreset,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub custom_theme: Option<CustomTheme>,

    // Time format settings
    pub timestamp_format: TimestampFormat,
    pub absolute_time_format: String,

    // Image settings (future inline image display)
    #[allow(dead_code)] // Reserved for future feature: Inline image display with ratatui-image
    pub inline_images: bool,
    #[allow(dead_code)] // Reserved for future feature: Inline image display protocol selection
    pub image_protocol: ImageProtocol,
    #[allow(dead_code)] // Reserved for future feature: Inline image max dimensions
    pub max_image_width: u32,
    #[allow(dead_code)] // Reserved for future feature: Inline image max dimensions
    pub max_image_height: u32,
    #[allow(dead_code)] // Reserved for future feature: Inline image caching
    pub image_cache_dir: PathBuf,

    // Aggregate tab key mappings (key -> tab name)
    pub tab_key_mappings: HashMap<char, String>,
    // Node key mappings for quick node switching (key -> node name)
    pub node_key_mappings: HashMap<char, String>,
    // Link key mappings for link selection mode (key -> link index)
    pub link_key_mappings: HashMap<char, usize>,
}

/// Theme presets
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemePreset {
    Dark,
    Light,
    Custom,
}

/// Custom theme colors
#[derive(Clone, Debug)]
pub struct CustomTheme {
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub foreground: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub background: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub primary: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub secondary: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub accent: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub error: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub success: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub warning: String,
    #[allow(dead_code)] // Reserved for future feature: Full custom color theming
    pub info: String,
}

/// Timestamp format options
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimestampFormat {
    Relative,
    Absolute,
}

/// Image protocol options
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
    Auto,
    #[allow(dead_code)] // Reserved for future feature: Sixel graphics protocol
    Sixel,
    #[allow(dead_code)] // Reserved for future feature: Kitty graphics protocol
    Kitty,
    #[allow(dead_code)] // Reserved for future feature: iTerm2 graphics protocol
    Iterm2,
    #[allow(dead_code)] // Reserved for future feature: Disable inline images
    None,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // General
            initial_view: View::TopicList,
            initial_tab: "tech".to_string(),
            initial_node: "python".to_string(),
            topics_per_page: 20,
            replies_per_page: 20,
            auto_refresh_interval: 0,
            key_sequence_timeout: 1000,

            // Nodes
            favorite_nodes: vec![
                ("python".to_string(), "Python".to_string()),
                ("programmer".to_string(), "程序员".to_string()),
                ("share".to_string(), "分享发现".to_string()),
                ("create".to_string(), "分享创造".to_string()),
                ("jobs".to_string(), "酷工作".to_string()),
                ("go".to_string(), "Go 编程语言".to_string()),
                ("rust".to_string(), "Rust 编程语言".to_string()),
                ("javascript".to_string(), "JavaScript".to_string()),
                ("linux".to_string(), "Linux".to_string()),
            ],
            quick_node_keys: {
                let mut map = HashMap::new();
                map.insert('1', "python".to_string());
                map.insert('2', "programmer".to_string());
                map.insert('3', "share".to_string());
                map.insert('4', "create".to_string());
                map.insert('5', "jobs".to_string());
                map.insert('6', "go".to_string());
                map.insert('7', "rust".to_string());
                map.insert('8', "javascript".to_string());
                map.insert('9', "linux".to_string());
                map
            },

            // Browser
            browser_command: None,

            // Theme
            theme: ThemePreset::Dark,
            custom_theme: None,

            // Time
            timestamp_format: TimestampFormat::Relative,
            absolute_time_format: "%Y-%m-%d %H:%M".to_string(),

            // Images
            inline_images: false,
            image_protocol: ImageProtocol::Auto,
            max_image_width: 800,
            max_image_height: 600,
            image_cache_dir: PathBuf::from("~/.cache/v2ex/images"),

            // Tab key mappings (default home row keys)
            tab_key_mappings: {
                let mut map = HashMap::new();
                map.insert('t', "tech".to_string());
                map.insert('c', "creative".to_string());
                map.insert('p', "play".to_string());
                map.insert('a', "apple".to_string());
                map.insert('j', "jobs".to_string());
                map.insert('d', "deals".to_string());
                map.insert('y', "city".to_string());
                map.insert('q', "qna".to_string());
                map.insert('i', "index".to_string());
                map
            },
            // Node key mappings (1-9 keys)
            node_key_mappings: {
                let mut map = HashMap::new();
                map.insert('1', "python".to_string());
                map.insert('2', "programmer".to_string());
                map.insert('3', "share".to_string());
                map.insert('4', "create".to_string());
                map.insert('5', "jobs".to_string());
                map.insert('6', "go".to_string());
                map.insert('7', "rust".to_string());
                map.insert('8', "javascript".to_string());
                map.insert('9', "linux".to_string());
                map
            },
            // Link key mappings (home row keys)
            link_key_mappings: {
                let mut map = HashMap::new();
                map.insert('a', 1);
                map.insert('o', 2);
                map.insert('e', 3);
                map.insert('u', 4);
                map.insert('i', 5);
                map.insert('d', 6);
                map.insert('h', 7);
                map.insert('t', 8);
                map.insert('n', 9);
                map.insert('s', 10);
                map
            },
        }
    }
}

impl Config {
    /// Get the configuration directory
    pub fn config_dir() -> Result<PathBuf> {
        let base_dirs = BaseDirs::new().context("Failed to get base directories")?;
        Ok(base_dirs.config_dir().join("v2ex"))
    }

    /// Get the config file path
    #[allow(dead_code)] // Reserved for future feature: Config reload and CLI config path display
    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.lisp"))
    }

    /// Create default configuration
    #[allow(dead_code)] // Reserved for future feature: Programmatic config creation
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    #[allow(dead_code)] // Reserved for future feature: Config validation before applying
    pub fn validate(&self) -> Result<()> {
        // TODO: Add validation logic
        Ok(())
    }
}

/// Runtime configuration state including keymaps
pub struct RuntimeConfig {
    pub config: Config,
    pub global_keymap: KeyMap,
    pub view_keymaps: HashMap<View, KeyMap>,
    pub mode_keymaps: HashMap<String, KeyMap>,
    pub action_registry: ActionRegistry,
}

impl RuntimeConfig {
    /// Create new runtime config with defaults
    pub fn new() -> Self {
        let config = Config::default();
        let action_registry = ActionRegistry::new();
        let global_keymap = Self::create_default_global_keymap();
        let view_keymaps = Self::create_default_view_keymaps();
        let mode_keymaps = Self::create_default_mode_keymaps();

        Self {
            config,
            global_keymap,
            view_keymaps,
            mode_keymaps,
            action_registry,
        }
    }

    fn create_default_global_keymap() -> KeyMap {
        use crossterm::event::{KeyCode, KeyModifiers};

        let mut keymap = KeyMap::new("global");

        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            },
            "quit-immediate",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::empty(),
            },
            "remove-from-history",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
            },
            "remove-from-history",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::empty(),
            },
            "history-back",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::empty(),
            },
            "history-forward",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::empty(),
            },
            "show-help",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::empty(),
            },
            "refresh",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::empty(),
            },
            "go-to-aggregate",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('m'),
                modifiers: KeyModifiers::empty(),
            },
            "go-to-notifications",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::empty(),
            },
            "go-to-profile",
        );
        keymap.bind(
            crate::keymap::Key {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::empty(),
            },
            "go-to-node-select",
        );

        keymap
    }

    fn create_default_view_keymaps() -> HashMap<View, KeyMap> {
        use crossterm::event::{KeyCode, KeyModifiers};

        let mut keymaps = HashMap::new();

        // Topic List
        let mut topic_list = KeyMap::new("topic-list");
        // Note: n/p navigation is provided by "browse" mode
        topic_list.bind(
            crate::keymap::Key {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::empty(),
            },
            "open-topic",
        );
        topic_list.bind(
            crate::keymap::Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
            },
            "open-topic",
        );
        topic_list.bind(
            crate::keymap::Key {
                code: KeyCode::Char('+'),
                modifiers: KeyModifiers::empty(),
            },
            "load-more-topics",
        );
        topic_list.bind(
            crate::keymap::Key {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::empty(),
            },
            "open-in-browser",
        );
        topic_list.bind(
            crate::keymap::Key {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::empty(),
            },
            "select-node",
        );
        // Note: </>/C-v/M-v navigation is provided by "browse" mode
        keymaps.insert(View::TopicList, topic_list);

        // Topic Detail
        let mut topic_detail = KeyMap::new("topic-detail");
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::empty(),
            },
            "toggle-replies",
        );
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::empty(),
            },
            "open-in-browser",
        );
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::empty(),
            },
            "enter-link-mode",
        );
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('+'),
                modifiers: KeyModifiers::empty(),
            },
            "load-more-replies",
        );
        // Cross-topic navigation (uppercase N/P)
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('N'),
                modifiers: KeyModifiers::empty(),
            },
            "next-topic",
        );
        topic_detail.bind(
            crate::keymap::Key {
                code: KeyCode::Char('P'),
                modifiers: KeyModifiers::empty(),
            },
            "previous-topic",
        );
        keymaps.insert(View::TopicDetail, topic_detail);

        // Aggregate view
        let mut aggregate = KeyMap::new("aggregate");
        aggregate.bind(
            crate::keymap::Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
            },
            "open-aggregate-item",
        );
        aggregate.bind(
            crate::keymap::Key {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::empty(),
            },
            "open-in-browser",
        );
        aggregate.bind(
            crate::keymap::Key {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::empty(),
            },
            "refresh-aggregate",
        );
        // Tab switching keys (t, c, k, a, j, d, y, z, i)
        for key in ['t', 'c', 'k', 'a', 'j', 'd', 'y', 'z', 'i'] {
            aggregate.bind(
                crate::keymap::Key {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::empty(),
                },
                "switch-tab",
            );
        }
        keymaps.insert(View::Aggregate, aggregate);

        // Notifications view
        let mut notifications = KeyMap::new("notifications");
        notifications.bind(
            crate::keymap::Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
            },
            "open-topic",
        );
        notifications.bind(
            crate::keymap::Key {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::empty(),
            },
            "open-in-browser",
        );
        notifications.bind(
            crate::keymap::Key {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::empty(),
            },
            "refresh",
        );
        keymaps.insert(View::Notifications, notifications);

        // Profile view
        let mut profile = KeyMap::new("profile");
        profile.bind(
            crate::keymap::Key {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::empty(),
            },
            "refresh",
        );
        keymaps.insert(View::Profile, profile);

        // Help view
        let mut help = KeyMap::new("help");
        help.bind(
            crate::keymap::Key {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::empty(),
            },
            "remove-from-history",
        );
        help.bind(
            crate::keymap::Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
            },
            "remove-from-history",
        );
        keymaps.insert(View::Help, help);

        keymaps
    }

    fn create_default_mode_keymaps() -> HashMap<String, KeyMap> {
        use crossterm::event::{KeyCode, KeyModifiers};

        let mut keymaps = HashMap::new();

        // Browse mode - shared navigation for list views (topic-list, aggregate)
        let mut browse = KeyMap::new("browse");
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::empty(),
            },
            "next-item",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::empty(),
            },
            "previous-item",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('<'),
                modifiers: KeyModifiers::empty(),
            },
            "first-item",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('>'),
                modifiers: KeyModifiers::empty(),
            },
            "last-item",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::CONTROL,
            },
            "page-down",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::ALT,
            },
            "page-up",
        );
        browse.bind(
            crate::keymap::Key {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::empty(),
            },
            "scroll-down",
        );
        keymaps.insert("browse".to_string(), browse);

        // Replies mode
        let mut replies = KeyMap::new("replies");
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::empty(),
            },
            "next-reply",
        );
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::empty(),
            },
            "previous-reply",
        );
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('+'),
                modifiers: KeyModifiers::empty(),
            },
            "load-more-replies",
        );
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('<'),
                modifiers: KeyModifiers::empty(),
            },
            "first-item",
        );
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('>'),
                modifiers: KeyModifiers::empty(),
            },
            "last-item",
        );
        // Cross-topic navigation in replies mode
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('N'),
                modifiers: KeyModifiers::empty(),
            },
            "next-topic",
        );
        replies.bind(
            crate::keymap::Key {
                code: KeyCode::Char('P'),
                modifiers: KeyModifiers::empty(),
            },
            "previous-topic",
        );
        keymaps.insert("replies".to_string(), replies);

        // Link selection mode
        let mut link_selection = KeyMap::new("link-selection");
        // Home row keys for link selection (all bound to same action, uses last_key)
        for key in ['a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's'] {
            link_selection.bind(
                crate::keymap::Key {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::empty(),
                },
                "link-select",
            );
        }
        link_selection.bind(
            crate::keymap::Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
            },
            "exit-link-mode",
        );
        keymaps.insert("link-selection".to_string(), link_selection);

        // Node selection mode
        let mut node_select = KeyMap::new("node-select");
        node_select.bind(
            crate::keymap::Key {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::empty(),
            },
            "next-item",
        );
        node_select.bind(
            crate::keymap::Key {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::empty(),
            },
            "previous-item",
        );
        node_select.bind(
            crate::keymap::Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
            },
            "select-current-node",
        );
        node_select.bind(
            crate::keymap::Key {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::empty(),
            },
            "toggle-completion-mode",
        );
        node_select.bind(
            crate::keymap::Key {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
            },
            "remove-from-history",
        );
        // Quick node selection with digit keys 1-9
        for digit in '1'..='9' {
            node_select.bind(
                crate::keymap::Key {
                    code: KeyCode::Char(digit),
                    modifiers: KeyModifiers::empty(),
                },
                "switch-node",
            );
        }
        keymaps.insert("node-select".to_string(), node_select);

        keymaps
    }

    /// Reset to default state
    #[allow(dead_code)] // Reserved for future feature: Hot config reload
    pub fn reset(&mut self) {
        self.config = Config::default();
        self.global_keymap = KeyMap::new("global");
        self.view_keymaps.clear();
        self.mode_keymaps.clear();
        // Don't reset action_registry - it has built-in actions
    }

    /// Build a keymap chain for the current state
    pub fn build_keymap_chain(&self, view: View, active_modes: &[String]) -> KeyMapChain {
        let mut chain = KeyMapChain::new();

        // Add global keymap (lowest priority)
        chain.push(std::rc::Rc::new(self.global_keymap.clone()));

        // Add view keymap
        if let Some(keymap) = self.view_keymaps.get(&view) {
            chain.push(std::rc::Rc::new(keymap.clone()));
        }

        // Add mode keymaps (highest priority)
        for mode in active_modes {
            if let Some(keymap) = self.mode_keymaps.get(mode) {
                chain.push(std::rc::Rc::new(keymap.clone()));
            }
        }

        chain
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export public types
pub use engine::ConfigEngine;
pub use loader::init_config;
