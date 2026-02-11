use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};

mod api;
mod app;
mod browser;
mod clipboard;
mod config;
mod keymap;
mod nodes;
mod state;
mod terminal;
mod ui;
mod util;
mod views;

use api::V2exClient;
use app::{App, View};
use terminal::TerminalManager;

fn print_help() {
    println!("v2ex-tui - A terminal UI viewer for V2EX");
    println!();
    println!("Usage: v2ex-tui [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help     Print help information");
    println!("  -v, --version  Print version information");
    println!();
    println!("Configuration:");
    println!("  Token file: ~/.config/v2ex/token.txt");
    println!();
    println!("  Get your Personal Access Token from:");
    println!("  https://www.v2ex.com/settings/tokens");
    println!();
    println!("Keyboard Shortcuts (Emacs/dired style):");
    println!("  n / p or ↓ / ↑ Move down/up (next/previous)");
    println!("  l / ← or r / → History back/forward (navigate view history)");
    println!("  Enter / t      Open selected topic/notification");
    println!(
        "  g              Refresh
   f              Enter link selection mode (in topic detail)
   w              Copy topic content or selected reply to clipboard (in topic detail)"
    );
    println!("  m              Notifications (messages)");
    println!("  u              Profile (user)");
    println!("  s              Select node from menu (Tab: manual input)");
    println!("  a              Aggregated topics view (RSS feeds)");
    println!("  1-9            Quick switch nodes (1:python, etc.) / Open links in topic view");
    println!(
        "  t              Open topic / Toggle replies view
   o              Open current topic in browser"
    );
    println!("  +              Load more topics");
    println!("  PageUp / PageDown  Load previous/next page of topics");
    println!("  N / P          Next/previous topic in detail view");
    println!("  < / >          Go to first/last item");
    println!("  ?              Help");
    println!("  q / Esc        Quit / Remove current view from history");
}

fn print_version() {
    println!("v2ex-tui 0.1.0");
}

async fn run_token_input(terminal: &mut TerminalManager) -> Result<Option<String>> {
    let mut app = App::new();
    app.view = View::TokenInput;
    app.ui_state.status_message = "Enter your V2EX token".to_string();

    loop {
        terminal.terminal().draw(|frame| app.render(frame))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        return Ok(None);
                    }
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        if !app.token_state.input.trim().is_empty() {
                            match app.token_state.save() {
                                Ok(_) => {
                                    return Ok(Some(app.token_state.input.trim().to_string()));
                                }
                                Err(e) => {
                                    app.ui_state.status_message =
                                        format!("Error saving token: {}", e);
                                }
                            }
                        } else {
                            app.ui_state.status_message = "Token cannot be empty".to_string();
                        }
                    }
                    KeyCode::Char(ch) => {
                        app.token_state.insert_char(ch);
                    }
                    KeyCode::Backspace => {
                        app.token_state.delete_char();
                    }
                    KeyCode::Left => {
                        app.token_state.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.token_state.move_cursor_right();
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn run_app(terminal: &mut TerminalManager, client: V2exClient) -> Result<()> {
    let mut app = App::new();

    // Initialize configuration
    let (config_engine, message) = config::init_config()?;

    // Show initialization message if any
    if let Some(msg) = message {
        app.ui_state.status_message = msg;
    }

    // Get runtime config for building keymap chains
    let runtime_config = config_engine.runtime_config();

    // Copy key mappings from config to app
    {
        let config = &runtime_config.borrow().config;
        app.tab_key_mappings = config.tab_key_mappings.clone();
        app.node_key_mappings = config.node_key_mappings.clone();
        app.link_key_mappings = config.link_key_mappings.clone();
        app.favorite_nodes = config.favorite_nodes.clone();
    }

    // Load initial view based on config
    {
        let config = &runtime_config.borrow().config;
        match config.initial_view {
            View::TopicList => {
                // Switch to initial node if specified
                if !config.initial_node.is_empty() {
                    app.node_state.switch_node(&config.initial_node);
                }
                app.load_topics(&client, false).await;
                app.navigate_to(View::TopicList);
            }
            View::Notifications => {
                app.load_notifications(&client).await;
                app.navigate_to(View::Notifications);
            }
            View::Profile => {
                app.load_profile(&client).await;
                app.navigate_to(View::Profile);
            }
            View::Aggregate => {
                // Switch to initial tab if specified
                if !config.initial_tab.is_empty() {
                    app.aggregate_state.switch_tab(&config.initial_tab);
                }
                app.load_aggregate(&client).await;
                app.navigate_to(View::Aggregate);
            }
            _ => {
                app.load_topics(&client, false).await;
                app.navigate_to(View::TopicList);
            }
        }
    }

    loop {
        terminal.terminal().draw(|frame| app.render(frame))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press {
                // Convert to our Key type
                let key: keymap::Key = key.into();

                // Store the key for actions that need to know the triggering key
                app.last_key = Some(key.clone());

                // Special handling: Node selection completion mode - insert printable characters
                if app.view == View::NodeSelect && app.node_state.is_completion_mode {
                    use crossterm::event::KeyCode;
                    if let KeyCode::Char(ch) = key.code {
                        // Insert character into completion input
                        app.node_state.insert_char(ch);
                        app.node_state.update_suggestions();
                        continue;
                    } else if key.code == KeyCode::Backspace {
                        // Handle backspace to delete character
                        app.node_state.delete_char();
                        app.node_state.update_suggestions();
                        continue;
                    }
                }

                // Build keymap chain for current state
                let active_modes: Vec<String> = if app.topic_state.link_input_state.is_active {
                    vec!["link-selection".to_string()]
                } else if app.view == View::TopicDetail && app.topic_state.show_replies {
                    // Only activate replies mode when in topic detail view
                    vec!["replies".to_string()]
                } else if app.view == View::TopicList || app.view == View::Aggregate {
                    // Browse mode for list views (shared navigation)
                    vec!["browse".to_string()]
                } else if app.view == View::NodeSelect {
                    // Node selection mode for navigating and selecting nodes
                    vec!["node-select".to_string()]
                } else {
                    vec![]
                };

                let chain = {
                    let runtime = runtime_config.borrow();
                    runtime.build_keymap_chain(app.view, &active_modes)
                };

                // Look up the key
                if let Some(binding) = chain.lookup(&key) {
                    match binding {
                        keymap::Binding::Action(action_name) => {
                            let action = {
                                let runtime = runtime_config.borrow();
                                runtime.action_registry.get(action_name).cloned()
                            };

                            if let Some(action) = action {
                                let should_exit = {
                                    let runtime = runtime_config.borrow();
                                    runtime
                                        .action_registry
                                        .execute(&action, &mut app, &client)
                                        .await?
                                };
                                if should_exit {
                                    break;
                                }
                            }
                        }
                        keymap::Binding::Prefix(_prefix_map) => {
                            // TODO: Handle key sequences
                            app.ui_state.status_message =
                                "Key sequences not yet implemented".to_string();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    // Try to load token
    let token = match V2exClient::load_token() {
        Ok(t) => t,
        Err(_) => {
            // Token not found, setup terminal and show token input view
            let mut manager = TerminalManager::new()?;
            let token_result = run_token_input(&mut manager).await;
            manager.shutdown()?;

            match token_result? {
                Some(t) => t,
                None => {
                    println!("Token input cancelled.");
                    std::process::exit(1);
                }
            }
        }
    };

    let client = V2exClient::new(token.clone());

    // Test API connection before starting TUI
    match client.get_member().await {
        Ok(member) => {
            println!("Connected to V2EX as: {}", member.username);
        }
        Err(e) => {
            // Token is invalid, but keep it for user to fix
            eprintln!("Error: Failed to connect to V2EX API: {}", e);
            eprintln!(
                "The token appears to be invalid. Please check your token in ~/.config/v2ex/token.txt"
            );
            eprintln!("You can get a new token from: https://www.v2ex.com/settings/tokens");
            std::process::exit(1);
        }
    }

    // Setup terminal
    let mut manager = TerminalManager::new()?;
    let result = run_app(&mut manager, client).await;
    manager.shutdown()?;

    result
}
