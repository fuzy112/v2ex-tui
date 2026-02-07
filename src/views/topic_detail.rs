use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{api::Topic, ui::Theme};

pub struct TopicDetailView;

impl TopicDetailView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
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

        // Content - prefer rendered HTML, fall back to raw content
        let content = topic
            .content_rendered
            .as_deref()
            .or(topic.content.as_deref())
            .unwrap_or("No content");
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

    #[allow(clippy::too_many_arguments)]
    pub fn render_split(
        &self,
        frame: &mut Frame,
        topic_area: Rect,
        replies_area: Rect,
        topic: &Topic,
        scroll: usize,
        replies: &[crate::api::Reply],
        list_state: &mut ListState,
        theme: &Theme,
    ) {
        self.render(frame, topic_area, topic, scroll, theme);
        self.render_replies(frame, replies_area, replies, list_state, theme);
    }

    fn render_replies(
        &self,
        frame: &mut Frame,
        area: Rect,
        replies: &[crate::api::Reply],
        list_state: &mut ListState,
        theme: &Theme,
    ) {
        let items: Vec<ListItem> = replies
            .iter()
            .enumerate()
            .map(|(index, reply)| {
                let is_selected = list_state.selected() == Some(index);

                let content_text = reply
                    .content_rendered
                    .as_deref()
                    .or(reply.content.as_deref())
                    .unwrap_or("No content");
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

                let author = reply
                    .member
                    .as_ref()
                    .map(|m| m.username.as_str())
                    .unwrap_or("Unknown");

                let header_line = Line::from(vec![
                    Span::styled(
                        format!("Reply #{} by ", index + 1),
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(author, Style::default().fg(theme.accent)),
                ]);

                let mut all_lines = vec![header_line];
                all_lines.extend(lines);
                all_lines.push(Line::from(""));

                ListItem::new(all_lines)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" Replies "),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_detail_view_new() {
        let _view = TopicDetailView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
