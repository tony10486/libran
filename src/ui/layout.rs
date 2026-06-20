use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::AppState;

use super::{left_panel, right_panel, status_bar};

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let body = chunks[0];
    let status = chunks[1];

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(body);

    left_panel::render(frame, panels[0], state);
    right_panel::render(frame, panels[1], state);
    status_bar::render(frame, status, state);
}
