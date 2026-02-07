// Placeholder for notifications view
pub struct NotificationsView;

impl NotificationsView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
        _notifications: &[crate::api::Notification],
        _selected: usize,
        _theme: &crate::ui::Theme,
    ) {
        // Implementation will be added in next step
    }
}
