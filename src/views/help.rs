// Placeholder for help view
pub struct HelpView;

impl HelpView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
        _theme: &crate::ui::Theme,
    ) {
        // Implementation will be added in next step
    }
}
