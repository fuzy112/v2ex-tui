//! Temporary bridge for the new keymap system
//!
//! This module provides a compatibility layer while we transition to the new
//! configurable keymap system. It will be replaced by the full config system.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::api::V2exClient;
use crate::app::App;
use crate::keymap::actions::ActionRegistry;
use crate::keymap::{Binding, Key, KeyMap, KeyMapChain};

/// Temporary event handler that uses the new keymap system
pub struct EventHandler {
    action_registry: ActionRegistry,
    keymap_chain: KeyMapChain,
    client: V2exClient,
}

impl EventHandler {
    pub fn new(client: &V2exClient) -> Self {
        let action_registry = ActionRegistry::new();
        let mut keymap_chain = KeyMapChain::new();

        // Create a basic keymap with default bindings
        let keymap = Self::create_default_keymap();
        keymap_chain.push(std::rc::Rc::new(keymap));

        Self {
            action_registry,
            keymap_chain,
            client: client.clone(),
        }
    }

    fn create_default_keymap() -> KeyMap {
        use KeyCode::*;
        use KeyModifiers as Mod;

        let mut keymap = KeyMap::new("default");

        // Basic bindings
        keymap.bind(
            Key {
                code: Char('c'),
                modifiers: Mod::CONTROL,
            },
            "quit-immediate",
        );

        keymap.bind(
            Key {
                code: Char('q'),
                modifiers: Mod::empty(),
            },
            "remove-from-history",
        );

        keymap.bind(
            Key {
                code: Esc,
                modifiers: Mod::empty(),
            },
            "remove-from-history",
        );

        keymap.bind(
            Key {
                code: Char('l'),
                modifiers: Mod::empty(),
            },
            "history-back",
        );

        keymap.bind(
            Key {
                code: Char('r'),
                modifiers: Mod::empty(),
            },
            "history-forward",
        );

        keymap.bind(
            Key {
                code: Char('n'),
                modifiers: Mod::empty(),
            },
            "next-topic",
        );

        keymap.bind(
            Key {
                code: Char('p'),
                modifiers: Mod::empty(),
            },
            "previous-topic",
        );

        keymap.bind(
            Key {
                code: Char('t'),
                modifiers: Mod::empty(),
            },
            "open-topic",
        );

        keymap.bind(
            Key {
                code: Enter,
                modifiers: Mod::empty(),
            },
            "open-topic",
        );

        keymap.bind(
            Key {
                code: Char('g'),
                modifiers: Mod::empty(),
            },
            "refresh",
        );

        keymap.bind(
            Key {
                code: Char('a'),
                modifiers: Mod::empty(),
            },
            "go-to-aggregate",
        );

        keymap.bind(
            Key {
                code: Char('m'),
                modifiers: Mod::empty(),
            },
            "go-to-notifications",
        );

        keymap.bind(
            Key {
                code: Char('u'),
                modifiers: Mod::empty(),
            },
            "go-to-profile",
        );

        keymap.bind(
            Key {
                code: Char('s'),
                modifiers: Mod::empty(),
            },
            "go-to-node-select",
        );

        keymap.bind(
            Key {
                code: Char('?'),
                modifiers: Mod::empty(),
            },
            "show-help",
        );

        keymap.bind(
            Key {
                code: Char('+'),
                modifiers: Mod::empty(),
            },
            "load-more-topics",
        );

        keymap.bind(
            Key {
                code: Char('v'),
                modifiers: Mod::CONTROL,
            },
            "page-down",
        );

        keymap.bind(
            Key {
                code: Char('v'),
                modifiers: Mod::ALT,
            },
            "page-up",
        );

        keymap
    }

    pub async fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        let key: Key = key.into();

        if let Some(binding) = self.keymap_chain.lookup(&key) {
            match binding {
                Binding::Action(action_name) => {
                    if let Some(action) = self.action_registry.get(action_name) {
                        return self
                            .action_registry
                            .execute(action, app, &self.client)
                            .await;
                    }
                }
                Binding::Prefix(_prefix_map) => {
                    // TODO: Handle key sequences
                    app.ui_state.status_message = "Key sequences not yet implemented".to_string();
                }
            }
        }

        Ok(false)
    }
}
