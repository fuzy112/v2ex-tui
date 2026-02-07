use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
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

#[allow(clippy::too_many_arguments)]
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
