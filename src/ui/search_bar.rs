use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

pub fn render_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let input = &state.search_input;
    let active = state.search_mode;

    let prompt_style = if active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if !input.is_empty() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input_style = if active {
        Style::default().fg(Color::White).bg(Color::Indexed(236))
    } else {
        Style::default().fg(Color::White)
    };

    let cursor = if active { "▎" } else { "" };
    let display_input = if !input.is_empty() {
        input.as_str()
    } else if active {
        ""
    } else {
        "검색어를 입력하세요 (/)"
    };
    let placeholder_style = if input.is_empty() && !active {
        Style::default().fg(Color::DarkGray)
    } else {
        input_style
    };

    let line = Line::from(vec![
        Span::styled(" / ", prompt_style),
        Span::styled(display_input.to_string(), placeholder_style),
        Span::styled(cursor.to_string(), Style::default().fg(Color::Cyan)),
    ]);

    frame.render_widget(Clear, area);
    let para = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(para, area);
}

pub fn render_search(frame: &mut Frame, area: Rect, input: &str) {
    let overlay = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, overlay);

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(Color::White).bg(Color::Black)),
        Span::styled("▎", Style::default().fg(Color::Cyan)),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, overlay);
}

pub fn render_add_file(frame: &mut Frame, area: Rect, input: &str) {
    let overlay = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, overlay);

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("a>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(Color::White).bg(Color::Black)),
        Span::styled("▎", Style::default().fg(Color::Yellow)),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, overlay);
}
