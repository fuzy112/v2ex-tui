use std::collections::HashMap;
use std::rc::Rc;

use crate::app::View;

use super::{actions::ActionRegistry, KeyMap};

/// Registry for managing all keymaps
pub struct KeyMapRegistry {
    /// Global keymap (available everywhere)
    global: Rc<KeyMap>,

    /// View-specific keymaps
    views: HashMap<View, Rc<KeyMap>>,

    /// Mode-specific keymaps (minor modes)
    modes: HashMap<String, Rc<KeyMap>>,

    /// Action registry
    action_registry: ActionRegistry,
}

impl KeyMapRegistry {
    /// Create a new keymap registry
    pub fn new() -> Self {
        Self {
            global: Rc::new(KeyMap::new("global")),
            views: HashMap::new(),
            modes: HashMap::new(),
            action_registry: ActionRegistry::new(),
        }
    }

    /// Get the global keymap
    pub fn global(&self) -> Rc<KeyMap> {
        self.global.clone()
    }

    /// Get a view keymap
    pub fn get_view(&self, view: View) -> Option<Rc<KeyMap>> {
        self.views.get(&view).cloned()
    }

    /// Get a mode keymap
    pub fn get_mode(&self, mode: &str) -> Option<Rc<KeyMap>> {
        self.modes.get(mode).cloned()
    }

    /// Set the global keymap
    pub fn set_global(&mut self, keymap: KeyMap) {
        self.global = Rc::new(keymap);
    }

    /// Register a view keymap
    pub fn register_view(&mut self, view: View, keymap: KeyMap) {
        self.views.insert(view, Rc::new(keymap));
    }

    /// Register a mode keymap
    pub fn register_mode(&mut self, mode: impl Into<String>, keymap: KeyMap) {
        self.modes.insert(mode.into(), Rc::new(keymap));
    }

    /// Get action registry
    pub fn action_registry(&self) -> &ActionRegistry {
        &self.action_registry
    }

    /// Reset all keymaps to empty state
    pub fn clear(&mut self) {
        self.global = Rc::new(KeyMap::new("global"));
        self.views.clear();
        self.modes.clear();
    }
}

impl Default for KeyMapRegistry {
    fn default() -> Self {
        Self::new()
    }
}
