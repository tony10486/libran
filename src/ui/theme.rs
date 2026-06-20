use ratatui::style::{Color, Modifier, Style};

pub fn bg_style() -> Style {
    Style::default().bg(Color::Black).fg(Color::Gray)
}

pub fn header_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black)
}

pub fn title_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
        .bg(Color::Black)
}

pub fn label_style() -> Style {
    Style::default().fg(Color::Cyan).bg(Color::Black)
}

pub fn meta_style() -> Style {
    Style::default().fg(Color::DarkGray).bg(Color::Black)
}

pub fn code_style() -> Style {
    Style::default().fg(Color::Yellow).bg(Color::Black)
}

pub fn key_style() -> Style {
    Style::default().fg(Color::Green).bg(Color::Black)
}

pub fn focus_style() -> Style {
    Style::default()
        .bg(Color::Cyan)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().fg(Color::Yellow).bg(Color::Black)
}

pub fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray).bg(Color::Black)
}

pub fn divider_style() -> Style {
    Style::default().fg(Color::DarkGray).bg(Color::Black)
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red).bg(Color::Black)
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Green).bg(Color::Black)
}
