use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{api::Member, ui::Theme};

pub struct ProfileView;

impl ProfileView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, member: &Member, theme: &Theme) {
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
}
