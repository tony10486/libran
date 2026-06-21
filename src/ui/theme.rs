use ratatui::style::{Color, Modifier, Style};

pub fn default_style() -> Style {
    Style::default().fg(Color::Gray).bg(Color::Black)
}

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
}

pub fn label_style() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn meta_style() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn code_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn key_style() -> Style {
    Style::default().fg(Color::Green)
}

pub fn focus_style() -> Style {
    Style::default()
        .bg(Color::DarkGray)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn dim_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn divider_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red)
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Green)
}
