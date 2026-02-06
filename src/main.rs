use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::ListState,
    Frame, Terminal,
};
use anyhow::Result;

mod api;
mod ui;

use api::{V2exClient, Topic, Reply, Notification, Member};
use ui::{Theme, render_topic_list, render_topic_detail, render_replies, 
         render_notifications, render_profile, render_help, render_loading, 
         render_error, render_status_bar, render_node_select};

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    TopicList,
    TopicDetail,
    Notifications,
    Profile,
    Help,
    NodeSelect,
}

#[derive(Debug)]
struct App {
    view: View,
    topics: Vec<Topic>,
    selected_topic: usize,
    current_topic: Option<Topic>,
    topic_replies: Vec<Reply>,
    notifications: Vec<Notification>,
    selected_notification: usize,
    profile: Option<Member>,
    current_node: String,
    page: i32,
    loading: bool,
    error: Option<String>,
    status_message: String,
    show_replies: bool,
    theme: Theme,
    // Scroll positions
    topic_scroll: usize,
    selected_reply: usize,
    // Node selection
    nodes: Vec<(String, String)>, // (name, title)
    selected_node: usize,
    // List state for replies
    replies_list_state: ListState,
}

impl App {
    fn new() -> Self {
        // Define available nodes
        let nodes = vec![
            ("python".to_string(), "Python".to_string()),
            ("programmer".to_string(), "程序员".to_string()),
            ("share".to_string(), "分享发现".to_string()),
            ("create".to_string(), "分享创造".to_string()),
            ("jobs".to_string(), "酷工作".to_string()),
            ("go".to_string(), "Go 编程语言".to_string()),
            ("rust".to_string(), "Rust 编程语言".to_string()),
            ("javascript".to_string(), "JavaScript".to_string()),
            ("linux".to_string(), "Linux".to_string()),
        ];
        
        Self {
            view: View::TopicList,
            topics: Vec::new(),
            selected_topic: 0,
            current_topic: None,
            topic_replies: Vec::new(),
            notifications: Vec::new(),
            selected_notification: 0,
            profile: None,
            current_node: "python".to_string(),
            page: 1,
            loading: false,
            error: None,
            status_message: "Press '?' for help".to_string(),
            show_replies: false,
            theme: Theme::default(),
            topic_scroll: 0,
            selected_reply: 0,
            nodes,
            selected_node: 0,
            replies_list_state: ListState::default(),
        }
    }

    async fn load_topics(&mut self, client: &V2exClient, append: bool) {
        self.loading = true;
        self.error = None;
        
        match client.get_node_topics(&self.current_node, self.page).await {
            Ok(mut new_topics) => {
                if append && self.page > 1 {
                    // Append new topics to existing ones
                    self.topics.append(&mut new_topics);
                    self.status_message = format!(
                        "Loaded {} more topics (total: {}) from {}",
                        new_topics.len(),
                        self.topics.len(),
                        self.current_node
                    );
                } else {
                    // Replace topics (first page or not appending)
                    self.topics = new_topics;
                    self.selected_topic = 0;
                    self.status_message = format!(
                        "Loaded {} topics from {}",
                        self.topics.len(),
                        self.current_node
                    );
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to load topics: {}", e));
            }
        }
        
        self.loading = false;
    }

    async fn load_topic_detail(&mut self, client: &V2exClient, topic_id: i64) {
        self.loading = true;
        self.error = None;
        
        match client.get_topic(topic_id).await {
            Ok(topic) => {
                self.current_topic = Some(topic);
                self.status_message = format!("Loaded topic {}", topic_id);
            }
            Err(e) => {
                self.error = Some(format!("Failed to load topic: {}", e));
            }
        }
        
        self.loading = false;
    }

    async fn load_topic_replies(&mut self, client: &V2exClient, topic_id: i64) {
        self.loading = true;
        self.error = None;
        
        match client.get_topic_replies(topic_id, 1).await {
            Ok(replies) => {
                self.topic_replies = replies;
                self.selected_reply = 0;
                self.replies_list_state.select(Some(0));
                self.status_message = format!("Loaded {} replies", self.topic_replies.len());
            }
            Err(e) => {
                self.error = Some(format!("Failed to load replies: {}", e));
            }
        }
        
        self.loading = false;
    }

    async fn load_notifications(&mut self, client: &V2exClient) {
        self.loading = true;
        self.error = None;
        
        match client.get_notifications(1).await {
            Ok(notifications) => {
                self.notifications = notifications;
                self.selected_notification = 0;
                self.status_message = format!("Loaded {} notifications", self.notifications.len());
            }
            Err(e) => {
                self.error = Some(format!("Failed to load notifications: {}", e));
            }
        }
        
        self.loading = false;
    }

    async fn load_profile(&mut self, client: &V2exClient) {
        self.loading = true;
        self.error = None;
        
        match client.get_member().await {
            Ok(member) => {
                self.profile = Some(member);
                self.status_message = "Loaded profile".to_string();
            }
            Err(e) => {
                self.error = Some(format!("Failed to load profile: {}", e));
            }
        }
        
        self.loading = false;
    }

    fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = (self.selected_topic + 1) % self.topics.len();
        }
    }

    fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_topic = if self.selected_topic == 0 {
                self.topics.len() - 1
            } else {
                self.selected_topic - 1
            };
        }
    }

    fn next_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification = (self.selected_notification + 1) % self.notifications.len();
        }
    }

    fn previous_notification(&mut self) {
        if !self.notifications.is_empty() {
            self.selected_notification = if self.selected_notification == 0 {
                self.notifications.len() - 1
            } else {
                self.selected_notification - 1
            };
        }
    }

    fn switch_node(&mut self, node: &str) {
        self.current_node = node.to_string();
        self.page = 1;
    }
    
    // Scroll methods
    fn scroll_topic_up(&mut self) {
        if self.topic_scroll >= 3 {
            self.topic_scroll -= 3;
        } else {
            self.topic_scroll = 0;
        }
    }
    
    fn scroll_topic_down(&mut self) {
        self.topic_scroll += 3; // Scroll 3 lines at a time for better performance
    }
    
    fn next_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = (self.selected_reply + 1) % self.topic_replies.len();
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }
    
    fn previous_reply(&mut self) {
        if !self.topic_replies.is_empty() {
            self.selected_reply = if self.selected_reply == 0 {
                self.topic_replies.len() - 1
            } else {
                self.selected_reply - 1
            };
            self.replies_list_state.select(Some(self.selected_reply));
        }
    }
    
    fn reset_scroll(&mut self) {
        self.topic_scroll = 0;
        self.selected_reply = 0;
        self.replies_list_state.select(Some(0));
    }
    
    // Node selection methods
    fn next_node(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_node = (self.selected_node + 1) % self.nodes.len();
        }
    }
    
    fn previous_node(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_node = if self.selected_node == 0 {
                self.nodes.len() - 1
            } else {
                self.selected_node - 1
            };
        }
    }
    
    fn select_current_node(&mut self) {
        if let Some((node_name, _)) = self.nodes.get(self.selected_node) {
            self.current_node = node_name.clone();
            self.page = 1;
        }
    }
    
    fn find_node_index(&self, node_name: &str) -> Option<usize> {
        self.nodes.iter().position(|(name, _)| name == node_name)
    }
    
    // Find current topic index in the topics list
    fn find_current_topic_index(&self) -> Option<usize> {
        if let Some(current_topic) = &self.current_topic {
            self.topics.iter().position(|topic| topic.id == current_topic.id)
        } else {
            None
        }
    }
    
    // Switch to next topic in detail view
    async fn switch_to_next_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.find_current_topic_index() {
            let next_index = (current_index + 1) % self.topics.len();
            if let Some(next_topic) = self.topics.get(next_index) {
                let topic_id = next_topic.id;
                self.current_topic = None;
                self.topic_replies.clear();
                self.reset_scroll();
                self.load_topic_detail(client, topic_id).await;
                self.load_topic_replies(client, topic_id).await;
                self.status_message = format!("Switched to next topic (#{})", next_index + 1);
            }
        }
    }
    
    // Switch to previous topic in detail view
    async fn switch_to_previous_topic(&mut self, client: &V2exClient) {
        if let Some(current_index) = self.find_current_topic_index() {
            let prev_index = if current_index == 0 {
                self.topics.len() - 1
            } else {
                current_index - 1
            };
            if let Some(prev_topic) = self.topics.get(prev_index) {
                let topic_id = prev_topic.id;
                self.current_topic = None;
                self.topic_replies.clear();
                self.reset_scroll();
                self.load_topic_detail(client, topic_id).await;
                self.load_topic_replies(client, topic_id).await;
                self.status_message = format!("Switched to previous topic (#{})", prev_index + 1);
            }
        }
    }
}

fn draw_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());

    match app.view {
        View::TopicList => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else {
                render_topic_list(
                    frame,
                    chunks[0],
                    &app.topics,
                    app.selected_topic,
                    &app.current_node,
                    &app.theme,
                );
            }
        }
        View::TopicDetail => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else if let Some(ref topic) = app.current_topic {
                if app.show_replies {
                    let split_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(chunks[0]);
                    render_topic_detail(frame, split_chunks[0], topic, app.topic_scroll, &app.theme);
                    render_replies(frame, split_chunks[1], &app.topic_replies, &mut app.replies_list_state, &app.theme);
                } else {
                    render_topic_detail(frame, chunks[0], topic, app.topic_scroll, &app.theme);
                }
            }
        }
        View::Notifications => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else {
                render_notifications(
                    frame,
                    chunks[0],
                    &app.notifications,
                    app.selected_notification,
                    &app.theme,
                );
            }
        }
        View::Profile => {
            if app.loading {
                render_loading(frame, chunks[0], &app.theme);
            } else if let Some(ref error) = app.error {
                render_error(frame, chunks[0], error, &app.theme);
            } else if let Some(ref profile) = app.profile {
                render_profile(frame, chunks[0], profile, &app.theme);
            }
        }
        View::Help => {
            render_help(frame, chunks[0], &app.theme);
        }
        View::NodeSelect => {
            render_node_select(
                frame,
                chunks[0],
                &app.nodes,
                app.selected_node,
                &app.current_node,
                &app.theme,
            );
        }
    }

    render_status_bar(frame, chunks[1], &app.status_message, &app.theme);
}

async fn run_app(
    terminal: &mut Terminal<impl Backend>,
    client: V2exClient,
) -> Result<()> {
    let mut app = App::new();
    
    // Load initial topics
    app.load_topics(&client, false).await;

    loop {
        terminal.draw(|frame| draw_ui(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        match app.view {
                            View::TopicList => break,
                            View::NodeSelect => {
                                app.view = View::TopicList;
                            }
                            _ => {
                                app.view = View::TopicList;
                                app.error = None;
                            }
                        }
                    }
                    KeyCode::Char('?') => {
                        app.view = View::Help;
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        match app.view {
                            View::NodeSelect => {
                                app.view = View::TopicList;
                            }
                            _ => {
                                if app.view != View::TopicList {
                                    app.view = View::TopicList;
                                    app.error = None;
                                }
                            }
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Down => {
                        match app.view {
                            View::TopicList => app.next_topic(),
                            View::Notifications => app.next_notification(),
                            View::NodeSelect => app.next_node(),
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    app.next_reply();
                                } else {
                                    app.scroll_topic_down();
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Up => {
                        match app.view {
                            View::TopicList => app.previous_topic(),
                            View::Notifications => app.previous_notification(),
                            View::NodeSelect => app.previous_node(),
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    app.previous_reply();
                                } else {
                                    app.scroll_topic_up();
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        match app.view {
                            View::TopicList => {
                                if let Some(topic) = app.topics.get(app.selected_topic) {
                                    let topic_id = topic.id;
                                    app.view = View::TopicDetail;
                                    app.show_replies = true;
                                    app.load_topic_detail(&client, topic_id).await;
                                    app.load_topic_replies(&client, topic_id).await;
                                }
                            }
                            View::Notifications => {
                                if let Some(notification) = app.notifications.get(app.selected_notification) {
                                    let topic_id = notification.extract_topic_id();
                                    let reply_id = notification.extract_reply_id();
                                    
                                    if let Some(topic_id) = topic_id {
                                        app.view = View::TopicDetail;
                                        app.show_replies = true;
                                        app.load_topic_detail(&client, topic_id).await;
                                        app.load_topic_replies(&client, topic_id).await;
                                        
                                        // Update status message
                                        if let Some(reply_id) = reply_id {
                                            app.status_message = format!("Jumping to topic {} (reply #{})", topic_id, reply_id);
                                        } else {
                                            app.status_message = format!("Jumping to topic {}", topic_id);
                                        }
                                    } else {
                                        app.status_message = "No topic link found in this notification".to_string();
                                    }
                                }
                            }
                            View::NodeSelect => {
                                app.select_current_node();
                                app.view = View::TopicList;
                                app.load_topics(&client, false).await;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('r') => {
                        match app.view {
                            View::TopicList => app.load_topics(&client, false).await,
                            View::TopicDetail => {
                                if let Some(ref topic) = app.current_topic {
                                    let topic_id = topic.id;
                                    app.load_topic_detail(&client, topic_id).await;
                                    app.load_topic_replies(&client, topic_id).await;
                                }
                            }
                            View::Notifications => app.load_notifications(&client).await,
                            View::Profile => app.load_profile(&client).await,
                            _ => {}
                        }
                    }
                    KeyCode::Char('m') => {
                        app.view = View::Notifications;
                        app.load_notifications(&client).await;
                    }
                    KeyCode::Char('u') => {
                        app.view = View::Profile;
                        app.load_profile(&client).await;
                    }
                    KeyCode::Char('s') => {
                        app.view = View::NodeSelect;
                        // Find current node in the list
                        if let Some(index) = app.find_node_index(&app.current_node) {
                            app.selected_node = index;
                        }
                    }
                    KeyCode::Char('t') => {
                        match app.view {
                            View::TopicList => {
                                if let Some(topic) = app.topics.get(app.selected_topic) {
                                    let topic_id = topic.id;
                                    app.view = View::TopicDetail;
                                    app.show_replies = true;
                                    app.load_topic_detail(&client, topic_id).await;
                                    app.load_topic_replies(&client, topic_id).await;
                                }
                            }
                            View::TopicDetail => {
                                app.show_replies = !app.show_replies;
                                app.reset_scroll();
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('N') => {
                        if app.view == View::TopicDetail {
                            app.switch_to_next_topic(&client).await;
                        }
                    }
                    KeyCode::Char('P') => {
                        if app.view == View::TopicDetail {
                            app.switch_to_previous_topic(&client).await;
                        }
                    }
                    KeyCode::Char('1') => {
                        app.switch_node("python");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('2') => {
                        app.switch_node("programmer");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('3') => {
                        app.switch_node("share");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('4') => {
                        app.switch_node("create");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('5') => {
                        app.switch_node("jobs");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('6') => {
                        app.switch_node("go");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('7') => {
                        app.switch_node("rust");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('8') => {
                        app.switch_node("javascript");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::Char('9') => {
                        app.switch_node("linux");
                        app.load_topics(&client, false).await;
                    }
                    KeyCode::PageDown => {
                        match app.view {
                            View::TopicList => {
                                app.page += 1;
                                app.load_topics(&client, true).await;
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    // Move 5 replies forward
                                    app.selected_reply = (app.selected_reply + 5).min(app.topic_replies.len() - 1);
                                } else {
                                    app.topic_scroll += 15; // Scroll 15 lines
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::PageUp => {
                        match app.view {
                            View::TopicList => {
                                if app.page > 1 {
                                    app.page -= 1;
                                app.load_topics(&client, false).await;
                                }
                            }
                            View::TopicDetail => {
                                if app.show_replies && !app.topic_replies.is_empty() {
                                    // Move 5 replies backward
                                    if app.selected_reply >= 5 {
                                        app.selected_reply -= 5;
                                    } else {
                                        app.selected_reply = 0;
                                    }
                                } else {
                                    if app.topic_scroll >= 15 {
                                        app.topic_scroll -= 15;
                                    } else {
                                        app.topic_scroll = 0;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('+') => {
                        match app.view {
                            View::TopicList => {
                                app.page += 1;
                                app.load_topics(&client, true).await;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('<') => {
                        match app.view {
                            View::TopicList => app.selected_topic = 0,
                            View::Notifications => app.selected_notification = 0,
                            _ => {}
                        }
                    }
                    KeyCode::Char('>') => {
                        match app.view {
                            View::TopicList => {
                                if !app.topics.is_empty() {
                                    app.selected_topic = app.topics.len() - 1;
                                }
                            }
                            View::Notifications => {
                                if !app.notifications.is_empty() {
                                    app.selected_notification = app.notifications.len() - 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

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
    println!("  n/p or ↓/↑     Move down/up (next/previous)");
    println!("  h/← or l/→     Navigate back/forward");
    println!("  Enter/t        Open selected topic/notification");
    println!("  r              Refresh");
    println!("  m              Notifications (messages)");
    println!("  u              Profile (user)");
    println!("  s              Select node from menu");
    println!("  1-9            Quick switch nodes (1:python, etc.)");
    println!("  t              Open topic / Toggle replies view");
    println!("  +              Load more topics");
    println!("  PageUp/Down    Load previous/next page of topics");
    println!("  N/P            Next/previous topic in detail view");
    println!("  </>            Go to first/last item");
    println!("  ?              Help");
    println!("  q/Esc          Quit");
}

fn print_version() {
    println!("v2ex-tui 0.1.0");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    for arg in &args[1..] {
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

    // Load token
    let token = match V2exClient::load_token() {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Error: V2EX token not found.");
            eprintln!("Please create a token at https://www.v2ex.com/settings/tokens");
            eprintln!("Then save it to: ~/.config/v2ex/token.txt");
            std::process::exit(1);
        }
    };

    let client = V2exClient::new(token);

    // Test API connection before starting TUI
    match client.get_member().await {
        Ok(member) => {
            println!("Connected to V2EX as: {}", member.username);
        }
        Err(e) => {
            eprintln!("Error: Failed to connect to V2EX API: {}", e);
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
