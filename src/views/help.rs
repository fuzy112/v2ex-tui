use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::Theme;

pub struct HelpView;

impl HelpView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_text = r#"
Keyboard Shortcuts:

Navigation (Emacs/dired style):
  n / ↓     - Move down (next) / Scroll down
  p / ↑     - Move up (previous) / Scroll up
  h / ←     - Go back
  l / →     - Open selected item

  PageUp    - Page up / Scroll up faster
  PageDown  - Page down / Scroll down faster
  < / >      - Go to first/last item

Actions:
  Enter     - Open selected topic/notification
  g         - Refresh current view
  m         - Go to notifications (messages)
  u         - Go to profile (user)

  q / Esc   - Quit / Go back

Topic List:
  s         - Select node from menu
  1-9       - Quick switch to node (1:python, 2:programmer, etc.)
  Enter / t - Open selected topic
  +         - Load more topics

Node Selection:
  n / p     - Navigate node list
  Enter     - Select node
  Tab       - Toggle manual input mode

Topic Detail:
  t         - Toggle replies view
  r         - Reply (if logged in)
  o         - Open in browser
  N / P     - Navigate between topics

Shortcuts:
  ?         - Show this help
  q         - Quit

Configuration:
  Token file: ~/.config/v2ex/token.txt
  Get token from: https://www.v2ex.com/settings/tokens
"#;

        let lines: Vec<Line> = help_text
            .lines()
            .map(|line| Line::styled(line, Style::default().fg(theme.foreground)))
            .collect();

        let help = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" Help "),
            )
            .scroll((0, 0));

        frame.render_widget(help, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_view_new() {
        let _view = HelpView::new();
        // Simple test to verify the view can be created
        assert!(true); // Placeholder assertion
    }
}
