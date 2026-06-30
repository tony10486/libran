// ── Widget Bar ───────────────────────────────────────────────────────────────
//
// 사이드바 상단에 표시되는 1줄 위젯 바.
// 각 위젯의 compact_bar() 요약을 '|' 구분자로 나열하여 표시.
// 예: | 4:23 PM | ⛅️ 23℃ | 📋 3/5 |

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::ui::theme;

/// 위젯 바를 렌더링합니다.
/// 사이드바 영역의 첫 1줄에 표시됩니다.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height == 0 || area.width < 4 {
        return;
    }

    let bars = state.widget_registry.compact_bars();

    if bars.is_empty() {
        return;
    }

    let mut spans: Vec<Span> = Vec::new();

    for (i, text) in bars.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                " │ ",
                Style::default().fg(theme::divider()).bg(theme::surface()),
            ));
        }
        spans.push(Span::styled(
            text,
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD)
                .bg(theme::surface()),
        ));
    }

    let line = Line::from(spans);
    let para = Paragraph::new(line).style(Style::default().bg(theme::surface()));
    frame.render_widget(para, area);
}
