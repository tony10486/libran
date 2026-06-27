use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};

use crate::app::AppState;
use crate::ui::theme;

pub fn render_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let input = &state.search_input;
    let active = state.search_mode;

    let prompt_style = if active {
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD)
    } else if !input.is_empty() {
        Style::default().fg(theme::accent_primary())
    } else {
        Style::default().fg(theme::dim())
    };

    let input_style = if active {
        Style::default()
            .fg(theme::focus_fg())
            .bg(theme::search_bg())
    } else {
        Style::default().fg(theme::title_fg())
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
        Style::default().fg(theme::dim())
    } else {
        input_style
    };

    let line = Line::from(vec![
        Span::styled(" / ", prompt_style),
        Span::styled(display_input.to_string(), placeholder_style),
        Span::styled(
            cursor.to_string(),
            Style::default().fg(theme::accent_secondary()),
        ),
    ]);

    frame.render_widget(Clear, area);
    let para = Paragraph::new(line).style(Style::default().bg(theme::bg()));
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
        Span::styled(
            "/",
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            input,
            Style::default().fg(theme::title_fg()).bg(theme::bg()),
        ),
        Span::styled("▎", Style::default().fg(theme::accent_secondary())),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(theme::fg()).bg(theme::bg()));
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
        Span::styled(
            "a>",
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            input,
            Style::default().fg(theme::title_fg()).bg(theme::bg()),
        ),
        Span::styled("▎", Style::default().fg(theme::accent_secondary())),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(theme::fg()).bg(theme::bg()));
    frame.render_widget(para, overlay);
}

pub fn render_command(frame: &mut Frame, area: Rect, input: &str) {
    let overlay = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, overlay);

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            ":",
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            input,
            Style::default().fg(theme::title_fg()).bg(theme::bg()),
        ),
        Span::styled("▎", Style::default().fg(theme::accent_secondary())),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(theme::fg()).bg(theme::bg()));
    frame.render_widget(para, overlay);
}
