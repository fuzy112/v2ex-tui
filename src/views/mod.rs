use ratatui::{backend::Backend, Frame};

use crate::app::App;

pub trait View {
    fn render<B: Backend>(&self, app: &App, frame: &mut Frame<B>);
}

pub mod topic_list;
pub mod topic_detail;
pub mod notifications;
pub mod profile;
pub mod node_select;
pub mod help;