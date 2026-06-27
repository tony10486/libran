pub mod export_dialog;
pub mod graph_panel;
pub mod help;
pub mod layout;
pub mod left_panel;
pub mod right_panel;
pub mod search_bar;
pub mod settings_panel;
pub mod stats_panel;
pub mod status_bar;
pub mod theme;

use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    layout::render(frame, state);

    if state.add_file_mode {
        search_bar::render_add_file(frame, frame.area(), &state.add_file_input);
    }

    if state.command_mode {
        search_bar::render_command(frame, frame.area(), &state.command_input);
    }

    if state.show_help {
        help::render(frame, frame.area(), state.help_page);
    }

    if state.show_export_dialog {
        export_dialog::render(frame, frame.area(), state);
    }

    if state.show_stats {
        if let Some(ref stats) = state.library_stats {
            stats_panel::render(frame, frame.area(), state, stats);
        }
    }

    if state.settings_panel_mode {
        settings_panel::render(frame, frame.area(), state);
    }
}
