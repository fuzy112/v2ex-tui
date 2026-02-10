//! Configuration engine with Ketos Lisp integration
//!
//! Provides builtin functions for configuring v2ex-tui via Lisp.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ketos::{rc_vec::RcString, Interpreter, Value};

use crate::app::View;
use crate::config::{RuntimeConfig, ThemePreset, TimestampFormat};
use crate::keymap::{Key, KeyMap};

/// Helper function to extract a string from a Ketos Value
fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.to_string()),
        _ => None,
    }
}

/// Configuration engine that manages the Ketos interpreter
pub struct ConfigEngine {
    interpreter: Interpreter,
    runtime_config: Rc<RefCell<RuntimeConfig>>,
}

impl ConfigEngine {
    /// Create a new configuration engine
    pub fn new() -> Self {
        let runtime_config = Rc::new(RefCell::new(RuntimeConfig::new()));
        let interpreter = Interpreter::new();

        let mut engine = Self {
            interpreter,
            runtime_config: runtime_config.clone(),
        };

        // Register all builtins
        engine.register_builtins();

        engine
    }

    /// Register builtin functions with the Ketos interpreter
    fn register_builtins(&mut self) {
        let scope = self.interpreter.scope();

        // Create bind function value
        let runtime = self.runtime_config.clone();
        let bind_fn = Value::new_foreign_fn(scope.add_name("bind"), move |_ctx, args| {
            if args.len() < 2 {
                return Err(ketos::Error::ExecError(ketos::exec::ExecError::expected(
                    "at least 2 arguments",
                    &args.len().into(),
                )));
            }

            let key_str = value_to_string(&args[0]).unwrap_or_default();
            let action = value_to_string(&args[1]).unwrap_or_default();

            let view = if args.len() > 2 {
                value_to_string(&args[2])
            } else {
                None
            };

            let mode = if args.len() > 3 {
                value_to_string(&args[3])
            } else {
                None
            };

            let result = if let Some(v) = view {
                bind_to_view(&runtime, &key_str, &action, &v)
            } else if let Some(m) = mode {
                bind_to_mode(&runtime, &key_str, &action, &m)
            } else {
                bind_global(&runtime, &key_str, &action)
            };

            match result {
                Ok(_) => Ok(Value::Unit),
                Err(e) => Ok(Value::String(RcString::from(e.to_string()))),
            }
        });
        scope.add_named_value("bind", bind_fn);

        // Create set! function value
        let runtime = self.runtime_config.clone();
        let set_fn = Value::new_foreign_fn(scope.add_name("set!"), move |_ctx, args| {
            if args.len() != 2 {
                return Err(ketos::Error::ExecError(ketos::exec::ExecError::expected(
                    "2 arguments",
                    &args.len().into(),
                )));
            }

            let key = value_to_string(&args[0]).unwrap_or_default();
            let value = args[1].clone();

            match set_config_value(&runtime, &key, &value) {
                Ok(_) => Ok(Value::Unit),
                Err(e) => Ok(Value::String(RcString::from(e.to_string()))),
            }
        });
        scope.add_named_value("set!", set_fn);

        // Create define-key function value
        let runtime = self.runtime_config.clone();
        let define_key_fn =
            Value::new_foreign_fn(scope.add_name("define-key"), move |_ctx, args| {
                if args.len() != 3 {
                    return Err(ketos::Error::ExecError(ketos::exec::ExecError::expected(
                        "3 arguments",
                        &args.len().into(),
                    )));
                }

                let keymap = value_to_string(&args[0]).unwrap_or_default();
                let key_str = value_to_string(&args[1]).unwrap_or_default();
                let action = value_to_string(&args[2]).unwrap_or_default();

                match define_key(&runtime, &keymap, &key_str, &action) {
                    Ok(_) => Ok(Value::Unit),
                    Err(e) => Ok(Value::String(RcString::from(e.to_string()))),
                }
            });
        scope.add_named_value("define-key", define_key_fn);
    }

    /// Get a reference to the runtime config
    pub fn runtime_config(&self) -> Rc<RefCell<RuntimeConfig>> {
        self.runtime_config.clone()
    }

    /// Load configuration from a file
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let code = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        self.interpreter
            .run_code(&code, None)
            .map_err(|e| anyhow::anyhow!("Config error: {}", e))?;

        Ok(())
    }

    /// Load configuration from a string
    pub fn load_string(&mut self, code: &str) -> Result<()> {
        self.interpreter
            .run_code(code, None)
            .map_err(|e| anyhow::anyhow!("Config error: {}", e))?;
        Ok(())
    }
}

impl Default for ConfigEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Bind a key to an action globally
fn bind_global(runtime: &Rc<RefCell<RuntimeConfig>>, key_str: &str, action: &str) -> Result<()> {
    let key = parse_key(key_str)?;
    let mut config = runtime.borrow_mut();
    config.global_keymap.bind(key, action);
    Ok(())
}

/// Bind a key to an action in a specific view
fn bind_to_view(
    runtime: &Rc<RefCell<RuntimeConfig>>,
    key_str: &str,
    action: &str,
    view_name: &str,
) -> Result<()> {
    let key = parse_key(key_str)?;
    let view = parse_view(view_name)?;
    let mut config = runtime.borrow_mut();

    config
        .view_keymaps
        .entry(view)
        .or_insert_with(|| KeyMap::new(view_name))
        .bind(key, action);

    Ok(())
}

/// Bind a key to an action in a specific mode
fn bind_to_mode(
    runtime: &Rc<RefCell<RuntimeConfig>>,
    key_str: &str,
    action: &str,
    mode_name: &str,
) -> Result<()> {
    let key = parse_key(key_str)?;
    let mut config = runtime.borrow_mut();

    config
        .mode_keymaps
        .entry(mode_name.to_string())
        .or_insert_with(|| KeyMap::new(mode_name))
        .bind(key, action);

    Ok(())
}

/// Define a key in a specific keymap by name
fn define_key(
    runtime: &Rc<RefCell<RuntimeConfig>>,
    keymap_name: &str,
    key_str: &str,
    action: &str,
) -> Result<()> {
    match keymap_name {
        "global" => bind_global(runtime, key_str, action),
        name if name.starts_with("view-") => {
            let view_name = &name[5..]; // Strip "view-" prefix
            bind_to_view(runtime, key_str, action, view_name)
        }
        name if name.starts_with("mode-") => {
            let mode_name = &name[5..]; // Strip "mode-" prefix
            bind_to_mode(runtime, key_str, action, mode_name)
        }
        _ => Err(anyhow::anyhow!("Unknown keymap: {}", keymap_name)),
    }
}

/// Set a configuration value
fn set_config_value(runtime: &Rc<RefCell<RuntimeConfig>>, key: &str, value: &Value) -> Result<()> {
    let mut config = runtime.borrow_mut();

    match key {
        "topics-per-page" => {
            if let Value::Integer(n) = value {
                config.config.topics_per_page = n.to_usize().unwrap_or(20);
            }
        }
        "replies-per-page" => {
            if let Value::Integer(n) = value {
                config.config.replies_per_page = n.to_usize().unwrap_or(20);
            }
        }
        "auto-refresh-interval" => {
            if let Value::Integer(n) = value {
                config.config.auto_refresh_interval = n.to_u64().unwrap_or(0);
            }
        }
        "key-sequence-timeout" => {
            if let Value::Integer(n) = value {
                config.config.key_sequence_timeout = n.to_u64().unwrap_or(1000);
            }
        }
        "theme" => {
            if let Value::String(s) = value {
                config.config.theme = match s.as_ref() {
                    "dark" | "Dark" => ThemePreset::Dark,
                    "light" | "Light" => ThemePreset::Light,
                    "custom" | "Custom" => ThemePreset::Custom,
                    _ => ThemePreset::Dark,
                };
            }
        }
        "timestamp-format" => {
            if let Value::String(s) = value {
                config.config.timestamp_format = match s.as_ref() {
                    "relative" | "Relative" => TimestampFormat::Relative,
                    "absolute" | "Absolute" => TimestampFormat::Absolute,
                    _ => TimestampFormat::Relative,
                };
            }
        }
        "inline-images" => {
            config.config.inline_images = matches!(value, Value::Bool(true));
        }
        "initial-view" => {
            if let Value::String(s) = value {
                config.config.initial_view = parse_view(s)?;
            }
        }
        "initial-tab" => {
            if let Value::String(s) = value {
                config.config.initial_tab = s.to_string();
            }
        }
        "initial-node" => {
            if let Value::String(s) = value {
                config.config.initial_node = s.to_string();
            }
        }
        _ => return Err(anyhow::anyhow!("Unknown config key: {}", key)),
    }

    Ok(())
}

/// Parse a key string in Emacs notation
/// Examples: "C-c", "M-v", "SPC", "RET"
pub fn parse_key(s: &str) -> Result<Key> {
    let mut chars = s.chars().peekable();
    let mut modifiers = crossterm::event::KeyModifiers::empty();

    // Parse modifiers
    while let Some(&ch) = chars.peek() {
        if ch == 'C' {
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= crossterm::event::KeyModifiers::CONTROL;
            } else {
                return Err(anyhow::anyhow!("Expected '-' after C"));
            }
        } else if ch == 'M' {
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= crossterm::event::KeyModifiers::ALT;
            } else {
                return Err(anyhow::anyhow!("Expected '-' after M"));
            }
        } else if ch == 'S' {
            // Check if this is "SPC" (space key) and not a Shift modifier
            let mut peek_chars = chars.clone();
            peek_chars.next(); // Consume 'S'
            if let Some(next_ch) = peek_chars.next() {
                if next_ch == '-' {
                    // This is Shift modifier (S-)
                    chars.next(); // Consume 'S'
                    chars.next(); // Consume '-'
                    modifiers |= crossterm::event::KeyModifiers::SHIFT;
                } else {
                    // This is likely "SPC" or another key starting with S
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Parse key code
    let remaining: String = chars.collect();
    let code = match remaining.as_str() {
        "SPC" => KeyCode::Char(' '),
        "RET" => KeyCode::Enter,
        "ESC" => KeyCode::Esc,
        "TAB" => KeyCode::Tab,
        "DEL" => KeyCode::Delete,
        "BACKSPACE" => KeyCode::Backspace,
        "LEFT" => KeyCode::Left,
        "RIGHT" => KeyCode::Right,
        "UP" => KeyCode::Up,
        "DOWN" => KeyCode::Down,
        "PAGEUP" | "PageUp" => KeyCode::PageUp,
        "PAGEDOWN" | "PageDown" => KeyCode::PageDown,
        "HOME" => KeyCode::Home,
        "END" => KeyCode::End,
        s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
        _ => return Err(anyhow::anyhow!("Unknown key: {}", remaining)),
    };

    Ok(Key { code, modifiers })
}

/// Parse a view name
pub fn parse_view(s: &str) -> Result<View> {
    match s {
        "topic-list" => Ok(View::TopicList),
        "topic-detail" => Ok(View::TopicDetail),
        "notifications" => Ok(View::Notifications),
        "profile" => Ok(View::Profile),
        "help" => Ok(View::Help),
        "node-select" => Ok(View::NodeSelect),
        "aggregate" => Ok(View::Aggregate),
        "token-input" => Ok(View::TokenInput),
        _ => Err(anyhow::anyhow!("Unknown view: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_engine_new() {
        let engine = ConfigEngine::new();
        let config = engine.runtime_config.borrow();

        // Verify default config values
        assert_eq!(config.config.topics_per_page, 20);
        assert_eq!(config.config.replies_per_page, 20);
        assert_eq!(config.config.auto_refresh_interval, 0);
    }

    #[test]
    fn test_parse_key() {
        // Test simple keys
        let key = parse_key("n").unwrap();
        assert_eq!(key.code, KeyCode::Char('n'));
        assert!(key.modifiers.is_empty());

        // Test control keys
        let key = parse_key("C-c").unwrap();
        assert_eq!(key.code, KeyCode::Char('c'));
        assert!(key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL));

        // Test alt keys
        let key = parse_key("M-v").unwrap();
        assert_eq!(key.code, KeyCode::Char('v'));
        assert!(key.modifiers.contains(crossterm::event::KeyModifiers::ALT));

        // Test special keys
        let key = parse_key("RET").unwrap();
        assert_eq!(key.code, KeyCode::Enter);

        let key = parse_key("ESC").unwrap();
        assert_eq!(key.code, KeyCode::Esc);

        let key = parse_key("SPC").unwrap();
        assert_eq!(key.code, KeyCode::Char(' '));
    }

    #[test]
    fn test_parse_view() {
        assert!(matches!(parse_view("topic-list"), Ok(View::TopicList)));
        assert!(matches!(parse_view("topic-detail"), Ok(View::TopicDetail)));
        assert!(matches!(
            parse_view("notifications"),
            Ok(View::Notifications)
        ));
        assert!(matches!(parse_view("profile"), Ok(View::Profile)));
        assert!(matches!(parse_view("help"), Ok(View::Help)));
        assert!(matches!(parse_view("node-select"), Ok(View::NodeSelect)));
        assert!(matches!(parse_view("aggregate"), Ok(View::Aggregate)));
        assert!(parse_view("unknown-view").is_err());
    }

    #[test]
    fn test_load_config_string() {
        let mut engine = ConfigEngine::new();

        // Load a simple config
        let config_code = r#"
            (set! "topics-per-page" 30)
            (set! "replies-per-page" 25)
            (bind "C-x" "quit-immediate")
        "#;

        let result = engine.load_string(config_code);
        assert!(result.is_ok(), "Failed to load config: {:?}", result);

        // Verify the config was applied
        let config = engine.runtime_config.borrow();
        assert_eq!(config.config.topics_per_page, 30);
        assert_eq!(config.config.replies_per_page, 25);

        // Check that the key was bound
        let key = parse_key("C-x").unwrap();
        match config.global_keymap.lookup(&key) {
            Some(crate::keymap::Binding::Action(action)) => {
                assert_eq!(action, "quit-immediate");
            }
            _ => panic!("Expected quit-immediate action to be bound to C-x"),
        }
    }

    #[test]
    fn test_define_key_view() {
        let mut engine = ConfigEngine::new();

        // Define a key in topic-list view
        let result = engine.load_string(
            r#"
            (define-key "view-topic-list" "j" "next-topic")
        "#,
        );

        assert!(result.is_ok());

        let config = engine.runtime_config.borrow();
        let key = parse_key("j").unwrap();
        match config
            .view_keymaps
            .get(&View::TopicList)
            .unwrap()
            .lookup(&key)
        {
            Some(crate::keymap::Binding::Action(action)) => {
                assert_eq!(action, "next-topic");
            }
            _ => panic!("Expected next-topic action to be bound to j in topic-list view"),
        }
    }

    #[test]
    fn test_define_key_mode() {
        let mut engine = ConfigEngine::new();

        // Define a key in replies mode
        let result = engine.load_string(
            r#"
            (define-key "mode-replies" "j" "next-reply")
        "#,
        );

        assert!(result.is_ok());

        let config = engine.runtime_config.borrow();
        let key = parse_key("j").unwrap();
        match config.mode_keymaps.get("replies").unwrap().lookup(&key) {
            Some(crate::keymap::Binding::Action(action)) => {
                assert_eq!(action, "next-reply");
            }
            _ => panic!("Expected next-reply action to be bound to j in replies mode"),
        }
    }

    #[test]
    fn test_keymap_chain_persists_across_builds() {
        let mut engine = ConfigEngine::new();

        // Load config that defines keys in topic-list view
        engine
            .load_string(
                r#"
            (define-key "view-topic-list" "x" "open-topic")
        "#,
            )
            .unwrap();

        // Build chain for topic-list view
        let chain1 = {
            let config = engine.runtime_config.borrow();
            config.build_keymap_chain(View::TopicList, &[])
        };

        // Verify key is bound
        let key = parse_key("x").unwrap();
        assert!(
            chain1.lookup(&key).is_some(),
            "Key should be bound in first chain"
        );

        // Build chain for different view
        let _chain2 = {
            let config = engine.runtime_config.borrow();
            config.build_keymap_chain(View::TopicDetail, &[])
        };

        // Build chain for topic-list view again
        let chain3 = {
            let config = engine.runtime_config.borrow();
            config.build_keymap_chain(View::TopicList, &[])
        };

        // Verify key is still bound
        assert!(
            chain3.lookup(&key).is_some(),
            "Key should still be bound after switching views"
        );
    }

    #[test]
    fn test_config_file_and_defaults_combined() {
        let mut engine = ConfigEngine::new();

        // 'n' is now in browse mode (shared navigation), not in topic-list view
        let config = engine.runtime_config.borrow();
        let key_n = parse_key("n").unwrap();
        let has_browse_binding = config
            .mode_keymaps
            .get("browse")
            .and_then(|km| km.lookup(&key_n))
            .is_some();
        drop(config);

        println!(
            "Has 'n' binding in browse mode before config load: {}",
            has_browse_binding
        );

        // Now load config that also binds a key
        engine
            .load_string(
                r#"
            (define-key "view-topic-list" "x" "open-topic")
        "#,
            )
            .unwrap();

        // Check both bindings exist
        let config = engine.runtime_config.borrow();
        let key_x = parse_key("x").unwrap();

        let topic_list_km = config
            .view_keymaps
            .get(&View::TopicList)
            .expect("Topic list keymap should exist");

        let has_browse = config
            .mode_keymaps
            .get("browse")
            .and_then(|km| km.lookup(&key_n))
            .is_some();
        let has_x = topic_list_km.lookup(&key_x).is_some();

        println!(
            "After config load - browse mode has 'n': {}, topic-list has 'x': {}",
            has_browse, has_x
        );

        // Both should be present - browse mode 'n' AND config file 'x'
        assert!(has_browse, "Browse mode 'n' binding should exist");
        assert!(has_x, "Config file 'x' binding should exist");
    }

    #[test]
    fn test_initial_tab_and_node_config() {
        let mut engine = ConfigEngine::new();

        // Load config with initial tab and node settings
        engine
            .load_string(
                r#"
            (set! "initial-tab" "creative")
            (set! "initial-node" "rust")
        "#,
            )
            .unwrap();

        // Verify the config was applied
        let config = engine.runtime_config.borrow();
        assert_eq!(config.config.initial_tab, "creative");
        assert_eq!(config.config.initial_node, "rust");
    }
}
