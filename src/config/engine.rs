//! Configuration engine with Ketos Lisp integration
//!
//! This is a simplified version that will be expanded.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use ketos::{Interpreter, Value};

use crate::app::View;
use crate::config::{CustomTheme, ImageProtocol, RuntimeConfig, ThemePreset, TimestampFormat};
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

        let engine = Self {
            interpreter,
            runtime_config: runtime_config.clone(),
        };

        // TODO: Register builtins when Ketos API is understood
        engine
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
            chars.next();
            if chars.next() == Some('-') {
                modifiers |= crossterm::event::KeyModifiers::SHIFT;
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
