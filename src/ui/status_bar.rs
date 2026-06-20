use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let status = format!(
        " {} | API: {} | {} 문헌 | {}",
        state.status_text,
        state.api_mode.as_str(),
        state.document_count,
        if state.api_mode.allows_api_calls() { "온라인" } else { "오프라인" }
    );

    let para = Paragraph::new(status);
    frame.render_widget(para, area);
}
