pub mod left_panel;
pub mod layout;
pub mod right_panel;
pub mod status_bar;
pub mod theme;

use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    layout::render(frame, state);
}
