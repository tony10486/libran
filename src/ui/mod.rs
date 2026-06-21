pub mod graph_panel;
pub mod help;
pub mod left_panel;
pub mod layout;
pub mod right_panel;
pub mod search_bar;
pub mod status_bar;
pub mod theme;

use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    layout::render(frame, state);

    if state.add_file_mode {
        search_bar::render_add_file(frame, frame.area(), &state.add_file_input);
    }

    if state.show_help {
        help::render(frame, frame.area());
    }
}
