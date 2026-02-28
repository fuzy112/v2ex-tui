use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    state::SearchState,
    ui::Theme,
    util::{format_timestamp, TimestampFormat},
};

pub struct SearchView;

impl SearchView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &SearchState,
        theme: &Theme,
        timestamp_format: TimestampFormat,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // Header with search input and sort indicator
        self.render_input_header(frame, chunks[0], state, theme);

        // Results list
        self.render_results(frame, chunks[1], state, theme, timestamp_format);

        // Status bar
        self.render_status_bar(frame, chunks[2], state, theme);
    }

    fn render_input_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &SearchState,
        theme: &Theme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(12)])
            .split(area);

        // Input box with cursor
        let cursor_char = if state.is_input_mode { '▌' } else { ' ' };
        let input_text = if state.cursor < state.query.len() {
            let before = &state.query[..state.cursor];
            let after = &state.query[state.cursor..];
            format!("{}{}{}", before, cursor_char, after)
        } else {
            format!("{}{}", state.query, cursor_char)
        };

        let input_style = if state.is_input_mode {
            Style::default()
                .fg(theme.primary)
                .bg(theme.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };

        let input_paragraph = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if state.is_input_mode {
                        Style::default().fg(theme.primary)
                    } else {
                        Style::default().fg(theme.muted)
                    })
                    .title(" Search (SOV2EX) "),
            )
            .style(input_style);

        frame.render_widget(input_paragraph, chunks[0]);

        // Sort indicator
        let sort_text = format!("[{}]", state.sort_by.display_name());
        let sort_style = Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD);
        let sort_paragraph = Paragraph::new(sort_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.muted)),
            )
            .style(sort_style);

        frame.render_widget(sort_paragraph, chunks[1]);
    }

    fn render_results(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &SearchState,
        theme: &Theme,
        timestamp_format: TimestampFormat,
    ) {
        let items: Vec<ListItem> = state
            .results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == state.selected {
                    Style::default()
                        .bg(theme.primary)
                        .fg(theme.background)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground)
                };

                let title = &result.source.title;
                let author = &result.source.member;
                let replies = result.source.replies;
                let topic_id = result.source.id;

                // Parse created timestamp
                let time_str = if let Ok(ts) = result.source.created.parse::<i64>() {
                    format_timestamp(ts, timestamp_format)
                } else {
                    result.source.created.clone()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", topic_id),
                        Style::default().fg(theme.secondary),
                    ),
                    Span::styled(title.to_string(), style),
                    Span::styled(
                        format!(" - @{} ", author),
                        Style::default().fg(theme.accent),
                    ),
                    Span::styled(format!("({})", replies), Style::default().fg(theme.muted)),
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
                    .title(format!(" Results [{}] ", state.results.len())),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(list, area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &SearchState, theme: &Theme) {
        let status_text = if state.results.is_empty() && !state.query.is_empty() {
            "Press Enter to search".to_string()
        } else {
            state.display_range()
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(theme.muted).bg(theme.background));

        frame.render_widget(status, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_view_new() {
        let _view = SearchView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
