use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::Theme;

pub struct NodeSelectView;

impl NodeSelectView {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)] // Required for all the parameters
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        nodes: &[(String, String)],
        selected: usize,
        current_node: &str,
        completion_input: &str,
        _completion_cursor: usize,
        is_completion_mode: bool,
        theme: &Theme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Input field
        let input_text = if is_completion_mode {
            format!("> {}", completion_input)
        } else {
            format!("Current node: {}", current_node)
        };

        let input = Paragraph::new(input_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Node Selection "),
        );

        frame.render_widget(input, chunks[0]);

        // Node list
        let items: Vec<ListItem> = nodes
            .iter()
            .enumerate()
            .map(|(i, (name, title))| {
                let style = if i == selected {
                    Style::default().bg(theme.primary).fg(theme.background)
                } else {
                    Style::default().fg(theme.foreground)
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", name),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(title, style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.secondary))
                    .title(" Nodes "),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.primary)
                    .fg(theme.background)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_select_view_new() {
        let _view = NodeSelectView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
