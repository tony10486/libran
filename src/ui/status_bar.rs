use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let sort_active = state.is_similarity_sorted();

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(
            &state.status_text,
            Style::default().fg(theme::fg()).bg(theme::bg()),
        ),
    ];

    if sort_active {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "│",
            Style::default().fg(theme::divider()).bg(theme::bg()),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "▸ 유사도 정렬",
            Style::default()
                .fg(theme::tag())
                .add_modifier(Modifier::BOLD)
                .bg(theme::bg()),
        ));
    }

    if state.is_processing {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "│",
            Style::default().fg(theme::divider()).bg(theme::bg()),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "처리 중...",
            Style::default()
                .fg(theme::warning())
                .add_modifier(Modifier::BOLD)
                .bg(theme::bg()),
        ));
    }

    let right_keys = [
        ("Tab", "패널"),
        ("j/k", "이동"),
        ("t", "태그"),
        ("r", "별점"),
        ("s", "유사도"),
        ("/", "검색"),
        ("?", "도움말"),
        ("q", "종료"),
    ];
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
            Style::default().fg(theme::accent_primary()).bg(theme::bg()),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            *label,
            Style::default().fg(theme::dim()).bg(theme::bg()),
        ));
    }

    let footer =
        Paragraph::new(Line::from(spans)).style(Style::default().fg(theme::fg()).bg(theme::bg()));
    frame.render_widget(footer, area);
}
