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

    pub fn render(&self, frame: &mut Frame, area: Rect, scroll: u16, theme: &Theme) {
        let help_text = r#"
Keyboard Shortcuts:

=== Global Navigation ===
  n / ↓     - Move down (next item)
  p / ↑     - Move up (previous item)
  l / ←     - History back (previous view)
  r / →     - History forward (next view)
  SPC       - Next item / scroll down
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
  q / Esc   - Go back / quit
  C-c       - Exit app immediately

=== Help ===
  n / ↓     - Scroll down
  p / ↑     - Scroll up
  PageDown  - Scroll down faster
  PageUp    - Scroll up faster
  q / Esc   - Close help

=== Topic List ===
  s         - Select node from menu
  1-9       - Quick switch node:
              1:python, 2:programmer, 3:分享发现,
              4:分享创造, 5:酷工作, 6:go,
              7:rust, 8:javascript, 9:linux
  Enter/t   - Open selected topic
  o         - Open selected topic in browser
  +         - Load more topics
  n (at end)- Auto-load more topics

=== Topic Detail ===
  t         - Toggle replies view
  o         - Open topic/reply in browser
  f         - Enter link selection mode
  w         - Copy selected reply, or topic content
  N / P     - Navigate between topics (auto-loads more)
  +         - Load more replies
  n / ↓     - Next reply (auto-loads at end)
  p / ↑     - Previous reply
  < / >     - Go to first/last reply

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
  o         - Open in browser
  g         - Refresh

=== Profile ===
  g         - Refresh profile

=== Smart Navigation ===
  • Most lists wrap around at the ends
  • 'n' at the end of a topic/reply list loads more content
  • The status bar shows "Already at the first/last" at boundaries

Configuration:
  Token file: ~/.config/v2ex/token.txt
              (or $XDG_CONFIG_HOME/v2ex/token.txt)
  Get token: https://www.v2ex.com/settings/tokens
"#;

        let lines: Vec<Line> = help_text
            .lines()
            .map(|line| Line::styled(line, Style::default().fg(theme.foreground)))
            .collect();

        let max_scroll = lines
            .len()
            .saturating_sub(area.height.saturating_sub(2) as usize) as u16;

        let help = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(" Help (n/p to scroll) "),
            )
            .scroll((scroll.min(max_scroll), 0));

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
