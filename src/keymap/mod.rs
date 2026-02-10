//! Keymap system for configurable keyboard input

pub mod actions;
pub mod handler;
pub mod registry;

// Re-export core types from the private module
pub use actions::{Action, ActionRegistry};
pub use handler::EventHandler;
pub use registry::KeyMapRegistry;

// Core keymap types
use std::collections::HashMap;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A key event representation for lookup
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// A binding in a keymap - either an action or a prefix for key sequences
#[derive(Clone, Debug)]
pub enum Binding {
    /// Execute an action
    Action(String), // Action name as string
    /// Prefix key - more keys expected
    Prefix(Rc<KeyMap>),
}

/// Sparse keymap with parent pointer (Emacs-style)
#[derive(Clone, Debug, Default)]
pub struct KeyMap {
    /// Local bindings (override parent)
    bindings: HashMap<Key, Binding>,
    /// Parent keymap for inheritance
    parent: Option<Rc<KeyMap>>,
    /// Name for debugging
    name: String,
}

impl KeyMap {
    /// Create a new empty keymap
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
            name: name.into(),
        }
    }

    /// Create a keymap with a parent
    pub fn with_parent(name: impl Into<String>, parent: Rc<KeyMap>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
            name: name.into(),
        }
    }

    /// Get the keymap name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Bind a key to an action
    pub fn bind(&mut self, key: Key, action: impl Into<String>) {
        self.bindings.insert(key, Binding::Action(action.into()));
    }

    /// Bind a key to a prefix keymap
    pub fn bind_prefix(&mut self, key: Key, prefix_map: Rc<KeyMap>) {
        self.bindings.insert(key, Binding::Prefix(prefix_map));
    }

    /// Unbind a key
    pub fn unbind(&mut self, key: &Key) -> Option<Binding> {
        self.bindings.remove(key)
    }

    /// Lookup a key in this keymap only
    pub fn lookup_local(&self, key: &Key) -> Option<&Binding> {
        self.bindings.get(key)
    }

    /// Lookup a key with parent fall-through
    pub fn lookup(&self, key: &Key) -> Option<&Binding> {
        self.bindings
            .get(key)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(key)))
    }

    /// Check if a key is bound locally
    pub fn is_bound(&self, key: &Key) -> bool {
        self.bindings.contains_key(key)
    }

    /// Get all local bindings
    pub fn bindings(&self) -> &HashMap<Key, Binding> {
        &self.bindings
    }
}

/// Priority-based keymap chain
/// Lookup order: temporary -> modes -> view -> global
#[derive(Clone, Debug, Default)]
pub struct KeyMapChain {
    /// Ordered by priority (highest first)
    keymaps: Vec<Rc<KeyMap>>,
}

impl KeyMapChain {
    /// Create an empty keymap chain
    pub fn new() -> Self {
        Self {
            keymaps: Vec::new(),
        }
    }

    /// Add a keymap to the chain (highest priority)
    pub fn push(&mut self, keymap: Rc<KeyMap>) {
        self.keymaps.push(keymap);
    }

    /// Remove and return the highest priority keymap
    pub fn pop(&mut self) -> Option<Rc<KeyMap>> {
        self.keymaps.pop()
    }

    /// Clear all keymaps
    pub fn clear(&mut self) {
        self.keymaps.clear();
    }

    /// Lookup a key in priority order
    /// Returns the first match found (includes parent keymap fall-through)
    pub fn lookup(&self, key: &Key) -> Option<&Binding> {
        for keymap in self.keymaps.iter().rev() {
            if let Some(binding) = keymap.lookup(key) {
                return Some(binding);
            }
        }
        None
    }

    /// Get all keymaps in the chain
    pub fn keymaps(&self) -> &[Rc<KeyMap>] {
        &self.keymaps
    }
}

/// Parse Emacs-style key notation
/// Examples: "C-c", "M-v", "C-x C-s", "SPC", "RET"
pub fn parse_key_sequence(_notation: &str) -> Result<Vec<Key>, String> {
    // TODO: Implement parsing
    // For now, return empty (will be implemented)
    Err("Key sequence parsing not yet implemented".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keymap_basic() {
        let mut map = KeyMap::new("test");
        let key = Key {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        };

        map.bind(key.clone(), "test-action");

        assert!(map.is_bound(&key));
        match map.lookup(&key) {
            Some(Binding::Action(action)) => assert_eq!(action, "test-action"),
            _ => panic!("Expected action binding"),
        }
    }

    #[test]
    fn test_keymap_inheritance() {
        let parent = Rc::new({
            let mut map = KeyMap::new("parent");
            let key = Key {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::empty(),
            };
            map.bind(key.clone(), "parent-action");
            map
        });

        let mut child = KeyMap::with_parent("child", parent);

        // Should inherit from parent
        let key = Key {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        };
        match child.lookup(&key) {
            Some(Binding::Action(action)) => assert_eq!(action, "parent-action"),
            _ => panic!("Expected inherited action"),
        }

        // Override in child
        child.bind(key.clone(), "child-action");
        match child.lookup(&key) {
            Some(Binding::Action(action)) => assert_eq!(action, "child-action"),
            _ => panic!("Expected child action"),
        }
    }
}
