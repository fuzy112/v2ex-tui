use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{api::Topic, ui::Theme, util::format_relative_time};

pub struct TopicListView;

impl TopicListView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
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
        let header = ratatui::widgets::Paragraph::new(Line::from(header_spans))
            .style(Style::default().bg(theme.background));
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
                let time_str = format_relative_time(topic.created);

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
                    Span::styled(format!(" • {}", time_str), Style::default().fg(theme.muted)),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_list_view_new() {
        let _view = TopicListView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
