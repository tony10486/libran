use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

use super::{left_panel, right_panel, status_bar};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::Black)),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(frame, chunks[0], state);

    if state.show_detail {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Length(1), Constraint::Min(1)])
            .split(chunks[1]);

        right_panel::render(frame, body[0], state);
        render_vdivider(frame, body[1]);
        right_panel::render_detail(frame, body[2], state);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Length(1), Constraint::Min(1)])
            .split(chunks[1]);

        left_panel::render(frame, body[0], state);
        render_vdivider(frame, body[1]);
        right_panel::render(frame, body[2], state);
    }

    status_bar::render(frame, chunks[2], state);

    if state.edit_mode {
        render_edit_overlay(frame, area, state);
    }
    if state.new_project_mode {
        render_new_project_overlay(frame, area, state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let online = state.api_mode.allows_api_calls();

    let header = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("Libran", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            if let Some(p) = state.projects.iter().find(|p| p.id == state.active_project_id) {
                p.name.clone()
            } else {
                "전체 문헌".to_string()
            },
            Style::default().fg(Color::Gray),
        ),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(state.document_count.to_string(), Style::default().fg(Color::Yellow)),
        Span::raw(" 문헌  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(state.api_mode.as_str(), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(
            if online { "● 온라인" } else { "● 오프라인" },
            Style::default().fg(if online { Color::Green } else { Color::DarkGray }),
        ),
    ]))
    .style(Style::default().bg(Color::Black));

    frame.render_widget(header, area);
}

fn render_vdivider(frame: &mut Frame, area: Rect) {
    for y in area.top()..area.bottom() {
        let divider = Paragraph::new("│")
            .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
        frame.render_widget(divider, Rect { x: area.x, y, width: 1, height: 1 });
    }
}

fn render_edit_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(60, 50, area);
    frame.render_widget(Clear, popup);

    let field_name = crate::app::dispatcher::EDIT_FIELD_NAMES
        .get(state.edit_field)
        .unwrap_or(&"?");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" 편집: {} ", field_name),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  필드: ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled(
                format!("{} ({}/{})", field_name, state.edit_field + 1, crate::app::dispatcher::EDIT_FIELD_NAMES.len()),
                Style::default().fg(Color::Cyan).bg(Color::Black),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Black)),
            Span::styled(state.edit_input.clone(), Style::default().fg(Color::White).bg(Color::Black)),
            Span::styled("▎", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 다음 필드  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 저장  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 취소", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_new_project_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(50, 25, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(" 새 프로젝트 ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  이름: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Black)),
            Span::styled(state.new_project_input.clone(), Style::default().fg(Color::White).bg(Color::Black)),
            Span::styled("▎", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 생성  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 취소", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().bg(Color::Black));
    frame.render_widget(para, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
