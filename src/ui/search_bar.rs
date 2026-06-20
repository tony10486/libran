use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

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

    let para = Paragraph::new(line).style(Style::default().bg(Color::Black));
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

    let para = Paragraph::new(line).style(Style::default().bg(Color::Black));
    frame.render_widget(para, overlay);
}
