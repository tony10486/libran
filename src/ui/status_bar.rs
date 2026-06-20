use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let online = state.api_mode.allows_api_calls();

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(&state.status_text, Style::default().fg(Color::Gray).bg(Color::Black)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::raw("  "),
        Span::styled("API:", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::raw(" "),
        Span::styled(state.api_mode.as_str(), Style::default().fg(Color::Cyan).bg(Color::Black)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::raw("  "),
        Span::styled(
            state.document_count.to_string(),
            Style::default().fg(Color::Yellow).bg(Color::Black),
        ),
        Span::styled(" 문헌", Style::default().fg(Color::Gray).bg(Color::Black)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::raw("  "),
        Span::styled(
            if online { "● 온라인" } else { "● 오프라인" },
            Style::default()
                .fg(if online { Color::Green } else { Color::DarkGray })
                .bg(Color::Black),
        ),
    ];

    if state.is_processing {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray).bg(Color::Black)));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "처리 중...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        ));
    }

    // Right-aligned shortcuts
    let right_keys = [("Tab", "패널"),
        ("j/k", "이동"),
        ("/", "검색"),
        ("?", "도움말"),
        ("q", "종료")];
    let right_len: usize = right_keys
        .iter()
        .map(|(k, l)| k.len() + l.len() + 3)
        .sum::<usize>()
        + (right_keys.len() - 1) * 2;

    let left_len: usize = spans.iter().map(|s| s.width()).sum();
    let padding = area.width as usize;
    let gap = padding.saturating_sub(left_len + right_len + 2);

    spans.push(Span::raw(" ".repeat(gap)));

    for (i, (key, label)) in right_keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(Color::Cyan).bg(Color::Black),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            *label,
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        ));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    frame.render_widget(footer, area);
}
