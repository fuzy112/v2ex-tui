use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{api::Topic, state::DetectedLink, ui::Theme};

pub struct TopicDetailView;

impl TopicDetailView {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        topic: &Topic,
        scroll: usize,
        detected_links: &[DetectedLink],
        is_link_mode_active: bool,
        parsed_content: Option<&str>,
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

        let content_text = if is_link_mode_active {
            // Use parsed content if available (from link detection)
            // Otherwise convert with consistent width
            if let Some(parsed) = parsed_content {
                parsed.to_string()
            } else {
                let width = std::cmp::min(100, area.width as usize);
                html2text::from_read(content.as_bytes(), width)
            }
        } else {
            html2text::from_read(content.as_bytes(), area.width as usize)
        };

        // Build text with link highlighting if link mode is active
        let content_display = if is_link_mode_active && !detected_links.is_empty() {
            self.build_highlighted_text(&content_text, detected_links, theme)
        } else {
            Text::from(content_text)
        };

        let content_para = Paragraph::new(content_display)
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

    fn build_highlighted_text<'a>(
        &self,
        text: &'a str,
        links: &[DetectedLink],
        theme: &Theme,
    ) -> Text<'a> {
        // Sort links by their start position
        let mut sorted_links: Vec<&DetectedLink> = links.iter().collect();
        sorted_links.sort_by_key(|link| link.text_range.start);

        let mut lines = Vec::new();
        let mut current_byte_pos = 0;
        let mut link_idx = 0;

        // Split text by newline, keeping the newline character to track positions accurately
        let line_iter = text.split_inclusive('\n');

        for line_with_newline in line_iter {
            let line_start = current_byte_pos;
            let line_len_with_newline = line_with_newline.len();
            let line_end = line_start + line_len_with_newline;

            // Remove trailing newline for display (ratatui handles line breaks)
            let line_content = line_with_newline
                .strip_suffix('\n')
                .unwrap_or(line_with_newline);

            let mut line_spans = Vec::new();
            let mut line_last_pos = line_start;

            // Process all links that start within this line
            while link_idx < sorted_links.len() {
                let link = sorted_links[link_idx];

                // If link starts at or after the end of this line, move to next line
                if link.text_range.start >= line_end {
                    break;
                }

                // Link should be within this line (URLs shouldn't span multiple lines)
                if link.text_range.start >= line_start && link.text_range.end <= line_end {
                    // Add text before link
                    if link.text_range.start > line_last_pos {
                        let start_in_line = link.text_range.start - line_start;
                        let end_in_line = line_last_pos - line_start;
                        line_spans.push(Span::raw(&line_content[end_in_line..start_in_line]));
                    }

                    // Add shortcut label
                    line_spans.push(Span::styled(
                        format!("[{}] ", link.shortcut),
                        Style::default()
                            .fg(theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ));

                    // Add link text
                    let link_start_in_line = link.text_range.start - line_start;
                    let link_end_in_line = link.text_range.end - line_start;
                    let link_text = if link_end_in_line <= line_content.len() {
                        &line_content[link_start_in_line..link_end_in_line]
                    } else {
                        // Link includes newline? Shouldn't happen but handle gracefully
                        &line_content[link_start_in_line..]
                    };
                    line_spans.push(Span::styled(
                        link_text,
                        Style::default()
                            .bg(theme.foreground)
                            .fg(theme.background)
                            .add_modifier(Modifier::REVERSED),
                    ));

                    line_last_pos = link.text_range.end;
                    link_idx += 1;
                } else {
                    // Link spans multiple lines or is malformed, skip it
                    link_idx += 1;
                }
            }

            // Add remaining text in line after last link
            if line_last_pos < line_end {
                let start_in_line = line_last_pos - line_start;
                if start_in_line < line_content.len() {
                    line_spans.push(Span::raw(&line_content[start_in_line..]));
                }
            }

            lines.push(Line::from(line_spans));
            current_byte_pos = line_end;
        }

        Text::from(lines)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_split(
        &self,
        frame: &mut Frame,
        topic_area: Rect,
        replies_area: Rect,
        topic: &Topic,
        scroll: usize,
        detected_links: &[DetectedLink],
        is_link_mode_active: bool,
        parsed_content: Option<&str>,
        replies: &[crate::api::Reply],
        list_state: &mut ListState,
        theme: &Theme,
    ) {
        self.render(
            frame,
            topic_area,
            topic,
            scroll,
            detected_links,
            is_link_mode_active,
            parsed_content,
            theme,
        );
        self.render_replies(
            frame,
            replies_area,
            replies,
            list_state,
            detected_links,
            is_link_mode_active,
            parsed_content,
            theme,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_replies(
        &self,
        frame: &mut Frame,
        area: Rect,
        replies: &[crate::api::Reply],
        list_state: &mut ListState,
        _detected_links: &[DetectedLink],
        _is_link_mode_active: bool,
        _parsed_content: Option<&str>,
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

                // TODO: Add link highlighting for replies (requires ownership fix in build_highlighted_text)
                // For now, use normal rendering even in link mode
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
