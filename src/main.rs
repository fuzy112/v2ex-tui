use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

mod api;
mod app;
mod event;
mod nodes;
mod ui;

use api::V2exClient;
use app::{App, View};
use event::EventHandler;

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
    println!("  h / ← or l / → Navigate back/forward");
    println!("  Enter / t      Open selected topic/notification");
    println!(
        "  g              Refresh
   r              Reply (in topic detail)"
    );
    println!("  m              Notifications (messages)");
    println!("  u              Profile (user)");
    println!("  s              Select node from menu (Tab: manual input)");
    println!("  1-9            Quick switch nodes (1:python, etc.)");
    println!(
        "  t              Open topic / Toggle replies view
   o              Open current topic in browser"
    );
    println!("  +              Load more topics");
    println!("  PageUp / PageDown  Load previous/next page of topics");
    println!("  N / P          Next/previous topic in detail view");
    println!("  < / >          Go to first/last item");
    println!("  ?              Help");
    println!("  q / Esc        Quit");
}

fn print_version() {
    println!("v2ex-tui 0.1.0");
}

async fn run_token_input(terminal: &mut Terminal<impl Backend>) -> Result<Option<String>> {
    let mut app = App::new();
    app.view = View::TokenInput;
    app.status_message = "Enter your V2EX token".to_string();

    loop {
        terminal.draw(|frame| app.render(frame))?;

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
                        if !app.token_input.trim().is_empty() {
                            match app.save_token() {
                                Ok(_) => {
                                    return Ok(Some(app.token_input.trim().to_string()));
                                }
                                Err(e) => {
                                    app.status_message = format!("Error saving token: {}", e);
                                }
                            }
                        } else {
                            app.status_message = "Token cannot be empty".to_string();
                        }
                    }
                    KeyCode::Char(ch) => {
                        app.insert_token_char(ch);
                    }
                    KeyCode::Backspace => {
                        app.delete_token_char();
                    }
                    KeyCode::Left => {
                        app.move_token_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_token_cursor_right();
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn run_app(terminal: &mut Terminal<impl Backend>, client: V2exClient) -> Result<()> {
    let mut app = App::new();
    let mut event_handler = EventHandler::new(&client);

    // Load initial topics
    app.load_topics(&client, false).await;

    loop {
        terminal.draw(|frame| app.render(frame))?;

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
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Run token input UI
            let token_result = run_token_input(&mut terminal).await;

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

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
            // Token is invalid, remove it
            if let Ok(config_dir) = V2exClient::config_dir() {
                let token_path = config_dir.join("token.txt");
                let _ = std::fs::remove_file(&token_path);
            }
            eprintln!("Error: Failed to connect to V2EX API: {}", e);
            eprintln!(
                "The token has been removed. Please run the application again with a valid token."
            );
            std::process::exit(1);
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, client).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
