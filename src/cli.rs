use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "v2ex-tui",
    about = "A CLI tool and TUI viewer for V2EX",
    version = "0.1.0"
)]
pub struct Cli {
    /// Output format (text or json)
    #[arg(short, long, value_enum, default_value = "text")]
    pub output: OutputFormat,

    /// Subcommand to execute (if none, starts TUI mode)
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List topics from a node
    List {
        /// Node name (e.g., python, programmer, share)
        #[arg(default_value = "python")]
        node: String,

        /// Page number
        #[arg(short, long, default_value = "1")]
        page: i32,

        /// Number of items to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show topic details
    Show {
        /// Topic ID
        id: i64,

        /// Include replies
        #[arg(short, long)]
        replies: bool,
    },

    /// Show topic replies
    Replies {
        /// Topic ID
        id: i64,

        /// Page number
        #[arg(short, long, default_value = "1")]
        page: i32,

        /// Number of items to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show notifications
    Notifications {
        /// Page number
        #[arg(short, long, default_value = "1")]
        page: i32,

        /// Number of items to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show user profile
    Profile,

    /// List available nodes
    Nodes {
        /// Filter nodes by keyword
        #[arg(short, long)]
        filter: Option<String>,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show aggregated topics from RSS feeds
    Aggregate {
        /// Tab name (index, hot, tech, creative, play, apple, jobs, deals, city, qna)
        #[arg(default_value = "index")]
        tab: String,

        /// Number of items to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

/// Print help message for TUI mode
#[allow(dead_code)] // Utility function for future help subcommand
pub fn print_tui_help() {
    println!("v2ex-tui - A terminal UI viewer for V2EX");
    println!();
    println!("Usage: v2ex-tui [OPTIONS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  list [NODE]     List topics from a node (default: python)");
    println!("  show <ID>       Show topic details");
    println!("  replies <ID>    Show topic replies");
    println!("  notifications   Show notifications");
    println!("  profile         Show user profile");
    println!("  nodes           List available nodes");
    println!("  aggregate       Show aggregated topics");
    println!();
    println!("Options:");
    println!("  -o, --output <FORMAT>  Output format: text or json [default: text]");
    println!("  -h, --help             Print help");
    println!("  -v, --version          Print version");
    println!();
    println!("Run without any command to start the interactive TUI.");
    println!();
    println!("Configuration:");
    println!("  Token file: ~/.config/v2ex/token.txt");
    println!("  Get your token from: https://www.v2ex.com/settings/tokens");
}

/// Print keyboard shortcuts for TUI mode
#[allow(dead_code)] // Utility function for future help subcommand
pub fn print_keyboard_shortcuts() {
    println!("Keyboard Shortcuts (Emacs/dired style):");
    println!("  n / p or ↓ / ↑  Move down/up (next/previous)");
    println!("  l / ← or r / →  History back/forward (navigate view history)");
    println!("  Enter / t       Open selected topic/notification");
    println!("  g               Refresh current view");
    println!("  f               Enter link selection mode (in topic detail)");
    println!("  w               Copy selected reply to clipboard (in topic detail)");
    println!("  m               Notifications (messages)");
    println!("  u               Profile (user)");
    println!("  s               Select node from menu (Tab: manual input)");
    println!("  a               Aggregated topics view (RSS feeds)");
    println!("  1-9             Quick switch nodes (1:python, etc.) / Open links in topic view");
    println!("  t               Open topic / Toggle replies view");
    println!("  o               Open current topic in browser");
    println!("  +               Load more topics");
    println!("  PageUp/PageDown Load previous/next page of topics");
    println!("  N / P           Next/previous topic in detail view");
    println!("  < / >           Go to first/last item");
    println!("  ?               Help");
    println!("  q / Esc         Quit / Remove current view from history");
}

/// Print version
#[allow(dead_code)] // Utility function for future version subcommand
pub fn print_version() {
    println!("v2ex-tui 0.1.0");
}

pub fn print_json<T: serde::Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

#[allow(dead_code)] // Utility function for future streaming output
pub fn print_json_line<T: serde::Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string(data)?;
    println!("{}", json);
    Ok(())
}

/// Limit a vector to a maximum number of items
#[allow(dead_code)] // Utility function kept for future use
pub fn limit_items<T>(items: &[T], limit: Option<usize>) -> &[T] {
    match limit {
        Some(n) if n < items.len() => &items[..n],
        _ => items,
    }
}
