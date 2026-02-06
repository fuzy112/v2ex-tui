use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, ListState},
    Frame,
};
use crate::api::{Topic, Reply, Notification, Member};

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

    frame.render_widget(list, area);
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
        .constraints([
            Constraint::Length(4),
            Constraint::Min(5),
        ])
        .split(area);

    // Header
    let author_name = topic.author_name();
    let node_name = topic.node_title();
    
    let header_lines = vec![
        Line::from(vec![
            Span::styled("Title: ", Style::default().fg(theme.primary)),
            Span::styled(&topic.title, Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Author: ", Style::default().fg(theme.primary)),
            Span::styled(author_name, Style::default().fg(theme.accent)),
            Span::styled(" | Node: ", Style::default().fg(theme.muted)),
            Span::styled(node_name, Style::default().fg(theme.secondary)),
            Span::styled(format!(" | Replies: {}", topic.replies), Style::default().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled("URL: ", Style::default().fg(theme.primary)),
            Span::styled(&topic.url, Style::default().fg(theme.muted)),
        ]),
        Line::from(""),
    ];

    let header = Paragraph::new(Text::from(header_lines))
        .block(
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
            let item_style = if is_selected {
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
            } else {
                Style::default()
            };
            
            let content_text = reply.content.as_deref().unwrap_or("No content");
            let content = html2text::from_read(
                content_text.as_bytes(),
                area.width.saturating_sub(4) as usize,
            );
            
            let lines: Vec<Line> = content
                .lines()
                .map(|line| {
                    if is_selected {
                        Line::styled(line.to_string(), Style::default().bg(theme.primary).fg(theme.background))
                    } else {
                        Line::from(line.to_string())
                    }
                })
                .collect();

            let author_name = reply.member.as_ref().map(|m| m.username.as_str()).unwrap_or("Unknown");
            let header = Line::from(vec![
                Span::styled(
                    format!("{:3}. ", index + 1),
                    if is_selected {
                        Style::default().fg(theme.background).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled(
                    format!("@{} ", author_name),
                    if is_selected {
                        Style::default().fg(theme.background).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled(
                    format!("(ID: {}) ", reply.id),
                    if is_selected {
                        Style::default().fg(theme.background).add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default().fg(theme.muted)
                    },
                ),
            ]);

            let mut all_lines = vec![header, Line::from("")];
            all_lines.extend(lines);
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(
                "─".repeat(area.width as usize)
            ));

            ListItem::new(Text::from(all_lines))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(format!(" Replies [{}/{}] ", list_state.selected().map(|s| s + 1).unwrap_or(0), replies.len())),
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
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
            } else {
                Style::default().fg(theme.foreground)
            };

            let raw_text = notif.text.clone();
            // Convert HTML to plain text
            let text = html2text::from_read(raw_text.as_bytes(), 80);
            
            let body = notif.payload
                .as_ref()
                .and_then(|p| p.extract_body())
                .map(|b| format!(" - {}", b))
                .unwrap_or_default();

            let author_name = notif.member.as_ref().map(|m| m.username.as_str()).unwrap_or("Unknown");
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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(format!(" Notifications [{}] ", notifications.len())),
        );

    frame.render_widget(list, area);
}

pub fn render_node_select(
    frame: &mut Frame,
    area: Rect,
    nodes: &[(String, String)],
    selected: usize,
    current_node: &str,
    theme: &Theme,
) {
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
                Span::styled(
                    format!("{:2}. ", i + 1),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(title.to_string(), style),
                Span::styled(
                    format!(" ({})", name),
                    Style::default().fg(theme.secondary),
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
                .title(" Select Node "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

pub fn render_profile(
    frame: &mut Frame,
    area: Rect,
    member: &Member,
    theme: &Theme,
) {
    let lines = vec![
        Line::from(vec![
            Span::styled("Username: ", Style::default().fg(theme.primary)),
            Span::styled(&member.username, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
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

    let profile = Paragraph::new(Text::from(lines))
        .block(
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

    let help = Paragraph::new(help_text)
        .block(
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

pub fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    message: &str,
    theme: &Theme,
) {
    let status = Paragraph::new(message)
        .style(Style::default().fg(theme.background).bg(theme.primary));

    frame.render_widget(status, area);
}
