use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{api::Notification, ui::Theme};

pub struct NotificationsView;

impl NotificationsView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
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
}
