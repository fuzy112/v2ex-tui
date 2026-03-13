use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};

mod api;
mod app;
mod browser;
mod cli;
mod cli_output;
mod clipboard;
mod keymap;
mod nodes;
mod state;
mod terminal;
mod ui;
mod util;
mod views;

use api::V2exClient;
use app::{App, View};
use cli::{Cli, Commands, OutputFormat};
use keymap::EventHandler;
use terminal::TerminalManager;

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

async fn run_cli(client: &V2exClient, cli: Cli) -> Result<()> {
    use cli_output::*;

    match cli.command.unwrap() {
        Commands::List { node, page, limit } => {
            let topics = client.get_node_topics(&node, page).await?;

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&topics)?;
                }
                OutputFormat::Text => {
                    println!("Topics from {} (page {}):\n", node, page);
                    print_topics(&topics, limit);
                    println!("\nTotal: {} topics", topics.len());
                }
            }
        }

        Commands::Show { id, replies } => {
            let topic = client.get_topic(id).await?;

            if replies {
                let replies_data = client.get_topic_replies(id, 1).await?;

                match cli.output {
                    OutputFormat::Json => {
                        #[derive(serde::Serialize)]
                        struct TopicWithReplies {
                            topic: api::Topic,
                            replies: Vec<api::Reply>,
                        }
                        let data = TopicWithReplies {
                            topic,
                            replies: replies_data,
                        };
                        cli::print_json(&data)?;
                    }
                    OutputFormat::Text => {
                        print_topic_detail(&topic);
                        println!("\n--- Replies ({} total) ---\n", topic.replies);
                        print_replies(&replies_data, None);
                    }
                }
            } else {
                match cli.output {
                    OutputFormat::Json => {
                        cli::print_json(&topic)?;
                    }
                    OutputFormat::Text => {
                        print_topic_detail(&topic);
                    }
                }
            }
        }

        Commands::Replies { id, page, limit } => {
            let replies = client.get_topic_replies(id, page).await?;

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&replies)?;
                }
                OutputFormat::Text => {
                    println!("Replies for topic {} (page {}):\n", id, page);
                    print_replies(&replies, limit);
                    println!("\nTotal: {} replies", replies.len());
                }
            }
        }

        Commands::Notifications { page, limit } => {
            let notifications = client.get_notifications(page).await?;

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&notifications)?;
                }
                OutputFormat::Text => {
                    println!("Notifications (page {}):\n", page);
                    print_notifications(&notifications, limit);
                    println!("\nTotal: {} notifications", notifications.len());
                }
            }
        }

        Commands::Profile => {
            let member = client.get_member().await?;

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&member)?;
                }
                OutputFormat::Text => {
                    println!("User Profile:\n");
                    print_member(&member);
                }
            }
        }

        Commands::Nodes { filter, limit } => {
            let nodes = find_nodes(filter.as_deref(), limit);

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&nodes)?;
                }
                OutputFormat::Text => {
                    if let Some(ref f) = filter {
                        println!("Nodes matching '{}':\n", f);
                    } else {
                        println!("Available nodes:\n");
                    }
                    print_nodes(&nodes, limit);
                    println!("\nTotal: {} nodes", nodes.len());
                }
            }
        }

        Commands::Aggregate { tab, limit } => {
            let items = client.get_rss_feed(&tab).await?;

            match cli.output {
                OutputFormat::Json => {
                    cli::print_json(&items)?;
                }
                OutputFormat::Text => {
                    println!("Aggregated topics from '{}' tab:\n", tab);
                    print_rss_items(&items, limit);
                    println!("\nTotal: {} items", items.len());
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::parse_args();

    // Check if we should run in TUI mode (no subcommand)
    let is_tui_mode = cli.command.is_none();

    // Try to load token
    let token = match V2exClient::load_token() {
        Ok(t) => t,
        Err(_) if is_tui_mode => {
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
        Err(e) => {
            eprintln!("Error: Failed to load token: {}", e);
            eprintln!("Please ensure you have a token in ~/.config/v2ex/token.txt");
            eprintln!("You can get a token from: https://www.v2ex.com/settings/tokens");
            std::process::exit(1);
        }
    };

    let client = V2exClient::new(token.clone());

    // Test API connection
    match client.get_member().await {
        Ok(member) => {
            if is_tui_mode {
                println!("Connected to V2EX as: {}", member.username);
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to connect to V2EX API: {}", e);
            eprintln!("The token appears to be invalid. Please check ~/.config/v2ex/token.txt");
            std::process::exit(1);
        }
    }

    if is_tui_mode {
        // Start TUI mode
        let mut manager = TerminalManager::new()?;
        let result = run_app(&mut manager, client).await;
        manager.shutdown()?;
        result
    } else {
        // Run CLI command
        run_cli(&client, cli).await
    }
}
