// ── Widget Panel UI ────────────────────────────────────────────────────────────
//
// settings_panel.rs와 동일한 오버레이 패턴으로 구현합니다.
// 화면 중앙 75% × 70% 팝업, 탭 바 + 콘텐츠 + 힌트 바.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::widget::{LineStyle, TextAlign, WidgetContent, WidgetStatus};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(78, 72, area);
    frame.render_widget(Clear, popup);

    let block = theme::create_theme_block(" 위젯  Widget Panel ");
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if state.widget_registry.is_empty() {
        let msg = Paragraph::new("위젯 없음. ~/.libran/widgets/ 에 위젯을 추가하세요.")
            .style(Style::default().fg(theme::dim()))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // 내부: 탭 바(1) + 콘텐츠(가변) + 힌트 바(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Min(1),    // content
            Constraint::Length(1), // hint bar
        ])
        .split(inner);

    render_tab_bar(frame, chunks[0], state);
    render_content(frame, chunks[1], state);
    render_hint_bar(frame, chunks[2], state);
}

// ── Tab Bar ───────────────────────────────────────────────────────────────────

fn render_tab_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let tabs = state.widget_registry.tab_labels();
    let active_idx = state.widget_registry.active_index;

    let mut spans = Vec::new();
    spans.push(Span::styled(" ", Style::default()));

    for (i, (name, badge)) in tabs.iter().enumerate() {
        let label = if let Some(b) = badge {
            format!(" {} ({}) ", name, b)
        } else {
            format!(" {} ", name)
        };

        if i == active_idx {
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(theme::bg())
                    .bg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(theme::dim())
                    .bg(theme::surface()),
            ));
        }
        if i + 1 < tabs.len() {
            spans.push(Span::styled("│", Style::default().fg(theme::divider())));
        }
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

// ── Content ───────────────────────────────────────────────────────────────────

fn render_content(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match state.widget_registry.active_content() {
        Some(c) => c,
        None => return,
    };

    match content.status {
        WidgetStatus::Loading => {
            let msg = Paragraph::new("  ⏳ 로딩 중...")
                .style(Style::default().fg(theme::dim()))
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }
        WidgetStatus::Error => {
            let err_msg = content
                .error_message
                .as_deref()
                .unwrap_or("알 수 없는 오류");
            let msg = Paragraph::new(format!("  ⚠  {}", err_msg))
                .style(Style::default().fg(theme::error()))
                .wrap(Wrap { trim: false });
            frame.render_widget(msg, area);
            return;
        }
        WidgetStatus::Ok => {}
    }

    render_widget_content(frame, area, content);
}

fn render_widget_content(frame: &mut Frame, area: Rect, content: &WidgetContent) {
    let mut ratatui_lines: Vec<Line> = Vec::new();

    // 선택적 제목
    if let Some(ref title) = content.title {
        ratatui_lines.push(Line::from(Span::styled(
            format!("  {}", title),
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        )));
        ratatui_lines.push(Line::from(Span::styled(
            "  ".to_string() + &"─".repeat(40),
            Style::default().fg(theme::divider()),
        )));
    }

    // 일반 lines
    for line in &content.lines {
        ratatui_lines.push(widget_line_to_ratatui(line));
    }

    // sections
    for section in &content.sections {
        ratatui_lines.push(Line::from(Span::styled(
            format!("  ▸ {}", section.header),
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD),
        )));
        for line in &section.lines {
            ratatui_lines.push(widget_line_to_ratatui(line));
        }
        ratatui_lines.push(Line::from(""));
    }

    let para = Paragraph::new(ratatui_lines)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn widget_line_to_ratatui(line: &crate::widget::WidgetLine) -> Line<'static> {
    let icon_prefix = line.icon.as_deref().map(|i| format!("{} ", i)).unwrap_or_default();
    let text = format!("{}{}", icon_prefix, line.text);

    let base_style = match line.style {
        LineStyle::Bold => Style::default().fg(theme::fg()).add_modifier(Modifier::BOLD),
        LineStyle::Dim => Style::default().fg(theme::dim()),
        LineStyle::Italic => Style::default().fg(theme::fg()).add_modifier(Modifier::ITALIC),
        LineStyle::Highlight => Style::default().fg(theme::bg()).bg(theme::accent_primary()),
        LineStyle::Error => Style::default().fg(theme::error()),
        LineStyle::Success => Style::default().fg(theme::success()),
        LineStyle::Warning => Style::default().fg(theme::warning()),
        LineStyle::Normal => Style::default().fg(theme::fg()),
    };

    // 커스텀 색상 오버라이드
    let style = if let Some(ref hex) = line.color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                base_style.fg(ratatui::style::Color::Rgb(r, g, b))
            } else {
                base_style
            }
        } else {
            base_style
        }
    } else {
        base_style
    };

    let alignment = match line.align {
        TextAlign::Center => Alignment::Center,
        TextAlign::Right => Alignment::Right,
        TextAlign::Left => Alignment::Left,
    };

    // 정렬은 Paragraph 레벨에서 처리하기 어려우므로 텍스트에 직접 반영
    let _ = alignment; // 향후 개선 가능

    Line::from(Span::styled(text, style))
}

// ── Hint Bar ──────────────────────────────────────────────────────────────────

fn render_hint_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = vec![
        Span::styled(" Tab", Style::default().fg(theme::key()).add_modifier(Modifier::BOLD)),
        Span::styled(":탭전환 ", Style::default().fg(theme::dim())),
        Span::styled("Esc", Style::default().fg(theme::key()).add_modifier(Modifier::BOLD)),
        Span::styled(":닫기 ", Style::default().fg(theme::dim())),
        Span::styled("1-9", Style::default().fg(theme::key()).add_modifier(Modifier::BOLD)),
        Span::styled(":직접선택", Style::default().fg(theme::dim())),
    ];

    // 활성 위젯의 커스텀 액션 표시
    if let Some(content) = state.widget_registry.active_content() {
        for action in &content.actions {
            spans.push(Span::styled(" │ ", Style::default().fg(theme::divider())));
            spans.push(Span::styled(
                action.key.to_string(),
                Style::default().fg(theme::key()).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(":{}", action.label),
                Style::default().fg(theme::dim()),
            ));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Layout helpers ────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
