use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::api::RssItem;
use crate::ui::Theme;

pub struct AggregateView;

impl AggregateView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        items: &[RssItem],
        selected: usize,
        current_tab: &str,
        theme: &Theme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        // Header with tab switch key binds
        let is_current = |tab: &str| tab == current_tab;
        let tab_style = |tab: &str| {
            if is_current(tab) {
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(theme.foreground)
            }
        };
        let key_style = |tab: &str| {
            if is_current(tab) {
                Style::default()
                    .fg(theme.background)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.accent)
            }
        };

        let header_spans = vec![
            Span::styled("Tabs: ", Style::default().fg(theme.muted)),
            Span::styled("t", key_style("tech")),
            Span::styled(":tech ", tab_style("tech")),
            Span::styled("c", key_style("creative")),
            Span::styled(":creative ", tab_style("creative")),
            Span::styled("k", key_style("play")),
            Span::styled(":play ", tab_style("play")),
            Span::styled("a", key_style("apple")),
            Span::styled(":apple ", tab_style("apple")),
            Span::styled("j", key_style("jobs")),
            Span::styled(":jobs ", tab_style("jobs")),
            Span::styled("d", key_style("deals")),
            Span::styled(":deals ", tab_style("deals")),
            Span::styled("y", key_style("city")),
            Span::styled(":city ", tab_style("city")),
            Span::styled("z", key_style("qna")),
            Span::styled(":qna ", tab_style("qna")),
            Span::styled("i", key_style("index")),
            Span::styled(":index", tab_style("index")),
        ];
        let header = ratatui::widgets::Paragraph::new(Line::from(header_spans))
            .style(Style::default().bg(theme.background));
        frame.render_widget(header, chunks[0]);

        let items_len = items.len();
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected {
                    Style::default()
                        .bg(theme.primary)
                        .fg(theme.background)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground)
                };

                let title = &item.title;
                let date = &item.date;

                let line = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", current_tab),
                        Style::default().fg(theme.secondary),
                    ),
                    Span::styled(title.to_string(), style),
                    Span::styled(format!(" ({})", date), Style::default().fg(theme.accent)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(format!(" Aggregated Topics [{}] ", items_len)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(list, chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_view_new() {
        let _view = AggregateView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
