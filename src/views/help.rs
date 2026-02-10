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

=== Global Navigation ===
  n / ↓     - Move down (next item)
  p / ↑     - Move up (previous item)
  l / ←     - History back (previous view)
  r / →     - History forward (next view)
  SPC       - Scroll down
  C-v       - Page down (Ctrl+v)
  M-v       - Page up (Alt+v)
  PageUp    - Page up / Scroll up faster
  PageDown  - Page down / Scroll down faster
  < / >     - Go to first/last item

=== Global Actions ===
  Enter     - Open selected item
  g         - Refresh current view
  m         - Go to notifications (messages)
  u         - Go to profile (user)
  a         - Go to aggregated topics (RSS feeds)
  ?         - Show this help
  q / Esc   - Quit / Remove current view from history
  C-c       - Exit app immediately

=== Topic List ===
  s         - Select node from menu
  1-9       - Quick switch node:
              1:python, 2:programmer, 3:分享发现,
              4:分享创造, 5:酷工作, 6:go,
              7:rust, 8:javascript, 9:linux
  Enter/t/l - Open selected topic
  +         - Load more topics
  n (at end)- Auto-load more topics

=== Topic Detail ===
  t         - Toggle replies view
  o         - Open topic/reply in browser
  f         - Enter link selection mode
  w         - Copy selected reply to clipboard
  N / P     - Navigate between topics (auto-loads more)
  +         - Load more replies
  n / ↓     - Next reply (auto-loads at end)
  p / ↑     - Previous reply
  1-9       - Open detected links by number

=== Link Selection Mode ===
  a,o,e,u,i,d,h,t,n,s - Type link shortcut letters
  Esc / q / C-g       - Cancel link selection

=== Aggregated Topics (RSS) ===
  t/c/k     - Switch to tech/creative/play tab
  a/j/d     - Switch to apple/jobs/deals tab
  y/z/i     - Switch to city/qna/index tab
  n / p     - Navigate topics
  Enter     - Open in app
  o         - Open in browser
  g         - Refresh

=== Node Selection ===
  n / p     - Navigate node list
  Enter     - Select node
  Tab       - Toggle manual input mode
  q (input) - Type 'q' character

=== Notifications ===
  n / p     - Navigate notifications
  Enter     - Open notification
  g         - Refresh

=== Profile ===
  g         - Refresh profile

=== Smart Navigation ===
  • Pressing 'n' at the end auto-loads more content
  • Shows "Already at first/last" instead of wrapping

Configuration:
  Token file: ~/.config/v2ex/token.txt
  Get token: https://www.v2ex.com/settings/tokens
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
