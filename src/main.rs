use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};

mod api;
mod app;
mod browser;
mod clipboard;
mod keymap;
mod nodes;
mod state;
mod terminal;
mod ui;
mod views;

use api::V2exClient;
use app::{App, View};
use keymap::EventHandler;
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
    let mut event_handler = EventHandler::new(&client);

    // Load initial aggregated topics
    app.load_aggregate(&client).await;

    loop {
        terminal.terminal().draw(|frame| app.render(frame))?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press && event_handler.handle_key(&mut app, key).await? {
                break;
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
