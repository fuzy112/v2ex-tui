// Placeholder for profile view
pub struct ProfileView;

impl ProfileView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
        _profile: &crate::api::Member,
        _theme: &crate::ui::Theme,
    ) {
        // Implementation will be added in next step
    }
}
