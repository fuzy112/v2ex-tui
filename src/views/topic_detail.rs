// Placeholder for topic_detail view - will be implemented next
pub struct TopicDetailView;

impl TopicDetailView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
        _topic: &crate::api::Topic,
        _scroll: usize,
        _theme: &crate::ui::Theme,
    ) {
        // Implementation will be added in next step
    }
}
