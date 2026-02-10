//! Configuration engine with Ketos Lisp integration

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use ketos::{Error as KetosError, FromValue, Interpreter, Scope, Value};

use crate::app::View;
use crate::config::{
    Config, CustomTheme, ImageProtocol, RuntimeConfig, ThemePreset, TimestampFormat,
};
use crate::keymap::{Key, KeyMap};

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

        engine.register_builtins();
        engine
    }

    /// Get a reference to the runtime config
    pub fn runtime_config(&self) -> Rc<RefCell<RuntimeConfig>> {
        self.runtime_config.clone()
    }

    /// Register all builtin functions
    fn register_builtins(&mut self) {
        let scope = self.interpreter.scope();
        let runtime = self.runtime_config.clone();

        // (set! symbol value) - Set configuration value
        Self::register_set(scope.clone(), runtime.clone());

        // (bind-global key action) - Bind key globally
        Self::register_bind_global(scope.clone(), runtime.clone());

        // (with-view view bindings...) - Define view keymap
        Self::register_with_view(scope.clone(), runtime.clone());

        // (with-mode mode bindings...) - Define mode keymap
        Self::register_with_mode(scope.clone(), runtime.clone());

        // (set-favorite-nodes list) - Set favorite nodes
        Self::register_set_favorite_nodes(scope.clone(), runtime.clone());

        // (set-quick-keys list) - Set quick key mappings
        Self::register_set_quick_keys(scope.clone(), runtime.clone());

        // (set-browser command) - Set browser command
        Self::register_set_browser(scope.clone(), runtime.clone());

        // (set-theme theme) - Set color theme
        Self::register_set_theme(scope.clone(), runtime.clone());

        // (set-custom-theme alist) - Set custom theme colors
        Self::register_set_custom_theme(scope.clone(), runtime.clone());

        // (set-timestamp-format format) - Set timestamp format
        Self::register_set_timestamp_format(scope.clone(), runtime.clone());

        // (set-absolute-time-format format) - Set absolute time format
        Self::register_set_absolute_time_format(scope.clone(), runtime.clone());

        // (set-inline-images bool) - Enable/disable inline images
        Self::register_set_inline_images(scope.clone(), runtime.clone());

        // (set-image-protocol protocol) - Set image protocol
        Self::register_set_image_protocol(scope.clone(), runtime.clone());
    }

    fn register_set(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set!", 2, move |args| {
            let symbol = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected symbol: {}", e).into()))?;
            let value = &args[1];

            let mut config = runtime.borrow_mut();

            match symbol.as_str() {
                "initial-view" => {
                    if let Ok(view) = View::from_value(value) {
                        config.config.initial_view = view;
                    }
                }
                "topics-per-page" => {
                    if let Ok(n) = usize::from_value(value) {
                        config.config.topics_per_page = n;
                    }
                }
                "replies-per-page" => {
                    if let Ok(n) = usize::from_value(value) {
                        config.config.replies_per_page = n;
                    }
                }
                "auto-refresh-interval" => {
                    if let Ok(n) = u64::from_value(value) {
                        config.config.auto_refresh_interval = n;
                    }
                }
                "key-sequence-timeout" => {
                    if let Ok(n) = u64::from_value(value) {
                        config.config.key_sequence_timeout = n;
                    }
                }
                _ => {
                    return Err(KetosError::Custom(
                        format!("Unknown setting: {}", symbol).into(),
                    ));
                }
            }

            Ok(Value::Unit)
        });
    }

    fn register_bind_global(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("bind-global", 2, move |args| {
            let key_str = String::from_value(&args[0])
                .map_err(|e| KetosError::Custom(format!("Expected string key: {}", e).into()))?;
            let action = args[1]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected action name: {}", e).into()))?;

            let key = parse_key(&key_str)
                .map_err(|e| KetosError::Custom(format!("Invalid key: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.global_keymap.bind(key, action.as_str());

            Ok(Value::Unit)
        });
    }

    fn register_with_view(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        // For now, this is a simplified version
        // Full implementation would evaluate body forms
        scope.add_fn_with_arity("with-view", 2, move |args| {
            let view_name = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected view name: {}", e).into()))?;

            let view = parse_view(&view_name)
                .map_err(|e| KetosError::Custom(format!("Unknown view: {}", e).into()))?;

            // TODO: Process bindings
            // For now, just create the keymap if it doesn't exist
            let mut config = runtime.borrow_mut();
            if !config.view_keymaps.contains_key(&view) {
                config
                    .view_keymaps
                    .insert(view, KeyMap::new(format!("{:?}", view)));
            }

            Ok(Value::Unit)
        });
    }

    fn register_with_mode(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("with-mode", 2, move |args| {
            let mode_name = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected mode name: {}", e).into()))?;

            // TODO: Process bindings
            let mut config = runtime.borrow_mut();
            if !config.mode_keymaps.contains_key(mode_name.as_str()) {
                config
                    .mode_keymaps
                    .insert(mode_name.to_string(), KeyMap::new(mode_name.as_str()));
            }

            Ok(Value::Unit)
        });
    }

    fn register_set_favorite_nodes(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-favorite-nodes", 1, move |args| {
            // TODO: Parse list of (name display-name) pairs
            // For now, just clear the list
            let mut config = runtime.borrow_mut();
            config.config.favorite_nodes.clear();
            Ok(Value::Unit)
        });
    }

    fn register_set_quick_keys(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-quick-keys", 1, move |args| {
            // TODO: Parse list of (key node) pairs
            let mut config = runtime.borrow_mut();
            config.config.quick_node_keys.clear();
            Ok(Value::Unit)
        });
    }

    fn register_set_browser(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-browser", 1, move |args| {
            let cmd = &args[0];

            let mut config = runtime.borrow_mut();

            if let Value::Bool(false) = cmd {
                // #f means system default
                config.config.browser_command = None;
            } else if let Ok(s) = String::from_value(cmd) {
                config.config.browser_command = Some(vec![s]);
            } else if let Ok(list) = <Vec<String>>::from_value(cmd) {
                config.config.browser_command = Some(list);
            }

            Ok(Value::Unit)
        });
    }

    fn register_set_theme(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-theme", 1, move |args| {
            let theme_name = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected theme name: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.config.theme = match theme_name.as_str() {
                "dark" => ThemePreset::Dark,
                "light" => ThemePreset::Light,
                "custom" => ThemePreset::Custom,
                _ => {
                    return Err(KetosError::Custom(
                        format!("Unknown theme: {}", theme_name).into(),
                    ))
                }
            };

            Ok(Value::Unit)
        });
    }

    fn register_set_custom_theme(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-custom-theme", 1, move |args| {
            // TODO: Parse alist of color settings
            let mut config = runtime.borrow_mut();
            config.config.custom_theme = Some(CustomTheme {
                foreground: "#ebdbb2".to_string(),
                background: "#282828".to_string(),
                primary: "#b8bb26".to_string(),
                secondary: "#fabd2f".to_string(),
                accent: "#83a598".to_string(),
                error: "#fb4934".to_string(),
                success: "#b8bb26".to_string(),
                warning: "#fabd2f".to_string(),
                info: "#83a598".to_string(),
            });
            Ok(Value::Unit)
        });
    }

    fn register_set_timestamp_format(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-timestamp-format", 1, move |args| {
            let format = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected format name: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.config.timestamp_format = match format.as_str() {
                "relative" => TimestampFormat::Relative,
                "absolute" => TimestampFormat::Absolute,
                _ => {
                    return Err(KetosError::Custom(
                        format!("Unknown timestamp format: {}", format).into(),
                    ))
                }
            };

            Ok(Value::Unit)
        });
    }

    fn register_set_absolute_time_format(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-absolute-time-format", 1, move |args| {
            let format = String::from_value(&args[0])
                .map_err(|e| KetosError::Custom(format!("Expected format string: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.config.absolute_time_format = format;

            Ok(Value::Unit)
        });
    }

    fn register_set_inline_images(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-inline-images", 1, move |args| {
            let enabled = bool::from_value(&args[0])
                .map_err(|e| KetosError::Custom(format!("Expected boolean: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.config.inline_images = enabled;

            Ok(Value::Unit)
        });
    }

    fn register_set_image_protocol(scope: Rc<Scope>, runtime: Rc<RefCell<RuntimeConfig>>) {
        scope.add_fn_with_arity("set-image-protocol", 1, move |args| {
            let protocol = args[0]
                .as_name()
                .map_err(|e| KetosError::Custom(format!("Expected protocol name: {}", e).into()))?;

            let mut config = runtime.borrow_mut();
            config.config.image_protocol = match protocol.as_str() {
                "auto" => ImageProtocol::Auto,
                "sixel" => ImageProtocol::Sixel,
                "kitty" => ImageProtocol::Kitty,
                "iterm2" => ImageProtocol::Iterm2,
                "none" => ImageProtocol::None,
                _ => {
                    return Err(KetosError::Custom(
                        format!("Unknown image protocol: {}", protocol).into(),
                    ))
                }
            };

            Ok(Value::Unit)
        });
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

/// Parse a key string in Emacs notation
/// Examples: "C-c", "M-v", "SPC", "RET"
fn parse_key(s: &str) -> Result<Key> {
    let mut chars = s.chars().peekable();
    let mut modifiers = KeyModifiers::empty();

    // Parse modifiers
    while let Some(&ch) = chars.peek() {
        if ch == 'C' {
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= KeyModifiers::CONTROL;
            } else {
                return Err(anyhow::anyhow!("Expected '-' after C"));
            }
        } else if ch == 'M' {
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= KeyModifiers::ALT;
            } else {
                return Err(anyhow::anyhow!("Expected '-' after M"));
            }
        } else if ch == 'S' {
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= KeyModifiers::SHIFT;
            } else {
                return Err(anyhow::anyhow!("Expected '-' after S"));
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
fn parse_view(s: &str) -> Result<View> {
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

/// Trait for converting Ketos values to/from our types
trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, KetosError>;
}

impl FromValue for View {
    fn from_value(value: &Value) -> Result<Self, KetosError> {
        let name = value.as_name()?;
        parse_view(&name).map_err(|e| KetosError::Custom(format!("Invalid view: {}", e).into()))
    }
}
