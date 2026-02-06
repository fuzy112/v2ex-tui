use crate::api::{Member, Notification, Reply, Topic};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

#[derive(Debug)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Yellow,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::Gray,
            error: Color::Red,
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_topic_list(
    frame: &mut Frame,
    area: Rect,
    topics: &[Topic],
    selected: usize,
    current_node: &str,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Header with node switch key binds
    let is_current = |node: &str| node == current_node;
    let node_style = |node: &str| {
        if is_current(node) {
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(theme.foreground)
        }
    };
    let key_style = |node: &str| {
        if is_current(node) {
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.accent)
        }
    };

    let header_spans = vec![
        Span::styled("Nodes: ", Style::default().fg(theme.muted)),
        Span::styled("1", key_style("python")),
        Span::styled(":python ", node_style("python")),
        Span::styled("2", key_style("programmer")),
        Span::styled(":programmer ", node_style("programmer")),
        Span::styled("3", key_style("share")),
        Span::styled(":share ", node_style("share")),
        Span::styled("4", key_style("create")),
        Span::styled(":create ", node_style("create")),
        Span::styled("5", key_style("jobs")),
        Span::styled(":jobs ", node_style("jobs")),
        Span::styled("6", key_style("go")),
        Span::styled(":go ", node_style("go")),
        Span::styled("7", key_style("rust")),
        Span::styled(":rust ", node_style("rust")),
        Span::styled("8", key_style("javascript")),
        Span::styled(":js ", node_style("javascript")),
        Span::styled("9", key_style("linux")),
        Span::styled(":linux ", node_style("linux")),
        Span::styled("s", Style::default().fg(theme.accent)),
        Span::styled(":more", Style::default().fg(theme.foreground)),
    ];
    let header =
        Paragraph::new(Line::from(header_spans)).style(Style::default().bg(theme.background));
    frame.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = topics
        .iter()
        .enumerate()
        .map(|(i, topic)| {
            let style = if i == selected {
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            let title = &topic.title;
            let replies = topic.replies;

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", current_node),
                    Style::default().fg(theme.secondary),
                ),
                Span::styled(title.to_string(), style),
                Span::styled(
                    format!(" ({} replies)", replies),
                    Style::default().fg(theme.accent),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(format!(" Topics [{}] ", topics.len())),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, chunks[1]);
}

pub fn render_topic_detail(
    frame: &mut Frame,
    area: Rect,
    topic: &Topic,
    scroll: usize,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(5)])
        .split(area);

    // Header
    let author_name = topic.author_name();
    let node_name = topic.node_title();

    let header_lines = vec![
        Line::from(vec![
            Span::styled("Title: ", Style::default().fg(theme.primary)),
            Span::styled(
                &topic.title,
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Author: ", Style::default().fg(theme.primary)),
            Span::styled(author_name, Style::default().fg(theme.accent)),
            Span::styled(" | Node: ", Style::default().fg(theme.muted)),
            Span::styled(node_name, Style::default().fg(theme.secondary)),
            Span::styled(
                format!(" | Replies: {}", topic.replies),
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("URL: ", Style::default().fg(theme.primary)),
            Span::styled(&topic.url, Style::default().fg(theme.muted)),
        ]),
        Line::from(""),
    ];

    let header = Paragraph::new(Text::from(header_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(" Topic "),
    );

    frame.render_widget(header, chunks[0]);

    // Content
    let content = topic.content.as_deref().unwrap_or("No content");
    let content_text = html2text::from_read(content.as_bytes(), area.width as usize);

    let content_para = Paragraph::new(content_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(" Content "),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));

    frame.render_widget(content_para, chunks[1]);
}

pub fn render_replies(
    frame: &mut Frame,
    area: Rect,
    replies: &[Reply],
    list_state: &mut ListState,
    theme: &Theme,
) {
    let items: Vec<ListItem> = replies
        .iter()
        .enumerate()
        .map(|(index, reply)| {
            let is_selected = list_state.selected() == Some(index);

            let content_text = reply.content.as_deref().unwrap_or("No content");
            let content = html2text::from_read(
                content_text.as_bytes(),
                area.width.saturating_sub(4) as usize,
            );

            let lines: Vec<Line> = content
                .lines()
                .map(|line| {
                    if is_selected {
                        Line::styled(
                            line.to_string(),
                            Style::default().bg(theme.primary).fg(theme.background),
                        )
                    } else {
                        Line::from(line.to_string())
                    }
                })
                .collect();

            let author_name = reply
                .member
                .as_ref()
                .map(|m| m.username.as_str())
                .unwrap_or("Unknown");
            let header = Line::from(vec![
                Span::styled(
                    format!("{:3}. ", index + 1),
                    if is_selected {
                        Style::default()
                            .fg(theme.background)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled(
                    format!("@{} ", author_name),
                    if is_selected {
                        Style::default()
                            .fg(theme.background)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled(
                    format!("(ID: {}) ", reply.id),
                    if is_selected {
                        Style::default()
                            .fg(theme.background)
                            .add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default().fg(theme.muted)
                    },
                ),
            ]);

            let mut all_lines = vec![header, Line::from("")];
            all_lines.extend(lines);
            all_lines.push(Line::from(""));
            all_lines.push(Line::from("─".repeat(area.width as usize)));

            ListItem::new(Text::from(all_lines))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(format!(
                    " Replies [{}/{}] ",
                    list_state.selected().map(|s| s + 1).unwrap_or(0),
                    replies.len()
                )),
        )
        .highlight_style(Style::default().bg(theme.primary).fg(theme.background))
        .start_corner(ratatui::layout::Corner::TopLeft);

    frame.render_stateful_widget(list, area, list_state);
}

pub fn render_notifications(
    frame: &mut Frame,
    area: Rect,
    notifications: &[Notification],
    selected: usize,
    theme: &Theme,
) {
    let items: Vec<ListItem> = notifications
        .iter()
        .enumerate()
        .map(|(i, notif)| {
            let style = if i == selected {
                Style::default().bg(theme.primary).fg(theme.background)
            } else {
                Style::default().fg(theme.foreground)
            };

            let raw_text = notif.text.clone();
            // Convert HTML to plain text
            let text = html2text::from_read(raw_text.as_bytes(), 80);

            let body = notif
                .payload
                .as_ref()
                .and_then(|p| p.extract_body())
                .map(|b| format!(" - {}", b))
                .unwrap_or_default();

            let author_name = notif
                .member
                .as_ref()
                .map(|m| m.username.as_str())
                .unwrap_or("Unknown");
            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", author_name),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(format!("{}{}", text, body), style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(format!(" Notifications [{}] ", notifications.len())),
    );

    frame.render_widget(list, area);
}

#[allow(clippy::too_many_arguments)]
pub fn render_node_select(
    frame: &mut Frame,
    area: Rect,
    nodes: &[(String, String)],
    selected: usize,
    current_node: &str,
    completion_input: &str,
    cursor_position: usize,
    is_node_completion_mode: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Node list
    let items: Vec<ListItem> = nodes
        .iter()
        .enumerate()
        .map(|(i, (name, title))| {
            let style = if i == selected {
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
                    .add_modifier(Modifier::BOLD)
            } else if name == current_node {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            let line = Line::from(vec![
                Span::styled(format!("{:2}. ", i + 1), Style::default().fg(theme.muted)),
                Span::styled(title.to_string(), style),
                Span::styled(format!(" ({})", name), Style::default().fg(theme.secondary)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Select Node "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, chunks[0]);

    // Node completion input field
    let mut input_spans = vec![Span::styled("Node: ", Style::default().fg(theme.primary))];

    let input_len = completion_input.chars().count();
    for (i, ch) in completion_input.chars().enumerate() {
        if is_node_completion_mode && i == cursor_position {
            input_spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(theme.primary).fg(theme.background),
            ));
        } else {
            input_spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme.foreground),
            ));
        }
    }

    if is_node_completion_mode && cursor_position == input_len {
        input_spans.push(Span::styled(
            " ",
            Style::default().bg(theme.primary).fg(theme.background),
        ));
    }

    let input_widget = Paragraph::new(Line::from(input_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if is_node_completion_mode {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.muted)
            })
            .title(" Node Completion (Tab to toggle) "),
    );

    frame.render_widget(input_widget, chunks[1]);
}

pub fn render_profile(frame: &mut Frame, area: Rect, member: &Member, theme: &Theme) {
    let lines = vec![
        Line::from(vec![
            Span::styled("Username: ", Style::default().fg(theme.primary)),
            Span::styled(
                &member.username,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(theme.primary)),
            Span::styled(member.id.to_string(), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("Location: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.location.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tagline: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.tagline.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("Bio: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.bio.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("Website: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.website.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.secondary),
            ),
        ]),
        Line::from(vec![
            Span::styled("GitHub: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.github.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.secondary),
            ),
        ]),
        Line::from(vec![
            Span::styled("Twitter: ", Style::default().fg(theme.primary)),
            Span::styled(
                member.twitter.as_deref().unwrap_or("Not set"),
                Style::default().fg(theme.secondary),
            ),
        ]),
    ];

    let profile = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(" Profile "),
    );

    frame.render_widget(profile, area);
}

pub fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = r#"
Keyboard Shortcuts:

Navigation (Emacs/dired style):
  n / ↓     - Move down (next) / Scroll down
  p / ↑     - Move up (previous) / Scroll up
  h / ←     - Go back
  l / →     - Open selected item
  g         - Go to top
  G         - Go to bottom
  PageUp    - Page up / Scroll up faster
  PageDown  - Page down / Scroll down faster

Actions:
  Enter     - Open selected topic/notification
  r         - Refresh current view
  m         - Go to notifications (messages)
  u         - Go to profile (user)
  /         - Search
  q / Esc   - Quit / Go back

Topic List:
  s         - Select node from menu
  1-9       - Quick switch to node (1:python, 2:programmer, etc.)
  Enter/t   - Open selected topic

Node Selection:
  n/p       - Navigate node list
  Enter     - Select node
  h/Esc     - Cancel

Topic Detail:
  t         - Toggle between content and replies
  n/p       - Scroll content/replies
  N/P       - Switch to next/previous topic
  PageUp/Dn - Scroll faster
  o         - Open in browser

Notifications:
  Enter     - Jump to linked topic
"#;

    let help = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(" Help "),
    );

    frame.render_widget(help, area);
}

pub fn render_loading(frame: &mut Frame, area: Rect, theme: &Theme) {
    let loading = Paragraph::new("Loading...")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        )
        .style(Style::default().fg(theme.accent));

    frame.render_widget(loading, area);
}

pub fn render_error(frame: &mut Frame, area: Rect, error: &str, theme: &Theme) {
    let error_widget = Paragraph::new(error)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.error))
                .title(" Error "),
        )
        .style(Style::default().fg(theme.error));

    frame.render_widget(error_widget, area);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    let status =
        Paragraph::new(message).style(Style::default().fg(theme.background).bg(theme.primary));

    frame.render_widget(status, area);
}

pub fn render_token_input(
    frame: &mut Frame,
    area: Rect,
    token: &str,
    cursor_position: usize,
    theme: &Theme,
) {
    use ratatui::layout::Alignment;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(15),
            Constraint::Percentage(20),
        ])
        .split(area);

    let input_area = centered_rect(80, 50, chunks[1]);

    let instructions = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Welcome to V2EX TUI!",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("A V2EX Personal Access Token is required to use this application."),
        Line::from(""),
        Line::from(vec![
            Span::styled("1. ", Style::default().fg(theme.accent)),
            Span::from("Go to "),
            Span::styled(
                "https://www.v2ex.com/settings/tokens",
                Style::default().fg(theme.secondary),
            ),
        ]),
        Line::from(vec![
            Span::styled("2. ", Style::default().fg(theme.accent)),
            Span::from("Create a new token"),
        ]),
        Line::from(vec![
            Span::styled("3. ", Style::default().fg(theme.accent)),
            Span::from("Paste it below and press Enter"),
        ]),
        Line::from(""),
    ];

    // Simple plain text input box
    let mut input_lines = vec![Line::from(vec![Span::styled(
        "Token: ",
        Style::default().fg(theme.primary),
    )])];

    // Show token as plain text with cursor
    let mut token_spans = vec![];
    let token_len = token.chars().count();

    for (i, ch) in token.chars().enumerate() {
        if i == cursor_position {
            // Cursor position - highlight
            token_spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(theme.primary).fg(theme.background),
            ));
        } else {
            token_spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme.foreground),
            ));
        }
    }

    // If cursor is at the end
    if cursor_position == token_len {
        token_spans.push(Span::styled(
            " ",
            Style::default().bg(theme.primary).fg(theme.background),
        ));
    }

    input_lines.push(Line::from(token_spans));
    input_lines.push(Line::from(""));
    input_lines.push(Line::from(vec![Span::styled(
        "Press Enter to save, Ctrl+C to quit",
        Style::default().fg(theme.muted),
    )]));

    let all_lines: Vec<Line> = instructions.into_iter().chain(input_lines).collect();

    let input_widget = Paragraph::new(Text::from(all_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Token Setup "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(input_widget, input_area);
}

pub fn render_reply_input(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    cursor_position: usize,
    theme: &Theme,
) {
    use ratatui::layout::Alignment;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Min(20),
            Constraint::Percentage(10),
        ])
        .split(area);

    let input_area = centered_rect(90, 70, chunks[1]);

    let instructions = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Reply to Topic",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Type your reply below. Markdown is supported.",
            Style::default().fg(theme.muted),
        )]),
    ];

    let mut input_lines = Vec::new();
    input_lines.push(Line::from(""));

    // Render content with cursor
    let content_len = content.chars().count();
    let mut content_spans = Vec::new();

    for (i, ch) in content.chars().enumerate() {
        if i == cursor_position {
            content_spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(theme.primary).fg(theme.background),
            ));
        } else {
            content_spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme.foreground),
            ));
        }
    }

    // If cursor is at the end
    if cursor_position == content_len {
        content_spans.push(Span::styled(
            " ",
            Style::default().bg(theme.primary).fg(theme.background),
        ));
    }

    input_lines.push(Line::from(content_spans));
    input_lines.push(Line::from(""));
    input_lines.push(Line::from(vec![Span::styled(
        "Press Enter to submit, Ctrl+C to cancel",
        Style::default().fg(theme.muted),
    )]));

    let all_lines: Vec<Line> = instructions.into_iter().chain(input_lines).collect();

    let input_widget = Paragraph::new(Text::from(all_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Reply "),
        )
        .alignment(Alignment::Center);

    frame.render_widget(input_widget, input_area);
}
