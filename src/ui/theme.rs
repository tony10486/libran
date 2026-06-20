use ratatui::style::{Color, Style};

pub fn default_style() -> Style {
    Style::default().fg(Color::White)
}

pub fn highlight_style() -> Style {
    Style::default().fg(Color::Yellow)
}
