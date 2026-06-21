use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::app::state::PanelFocus;

use super::{graph_panel, left_panel, right_panel, search_bar, status_bar};

struct RightArea {
    search: Rect,
    list: Rect,
}

fn split_right_with_search(area: Rect, _state: &AppState) -> RightArea {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    RightArea {
        search: chunks[0],
        list: chunks[1],
    }
}

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    frame.render_widget(
        Paragraph::new("").style(Style::default().fg(Color::Gray).bg(Color::Black)),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(frame, chunks[0], state);

    if state.graph_state.is_some() && state.active_panel == PanelFocus::Graph {
        graph_panel::render(frame, chunks[1], state);
    } else if state.show_detail {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Length(1), Constraint::Min(1)])
            .split(chunks[1]);

        right_panel::render(frame, body[0], state);
        render_vdivider(frame, body[1]);
        let right_area = split_right_with_search(body[2], state);
        right_panel::render(frame, right_area.list, state);
        search_bar::render_bar(frame, right_area.search, state);
        right_panel::render_detail(frame, body[2], state);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Length(1), Constraint::Min(1)])
            .split(chunks[1]);

        left_panel::render(frame, body[0], state);
        render_vdivider(frame, body[1]);
        let right_area = split_right_with_search(body[2], state);
        search_bar::render_bar(frame, right_area.search, state);
        right_panel::render(frame, right_area.list, state);
    }

    status_bar::render(frame, chunks[2], state);

    if state.edit_mode {
        render_edit_overlay(frame, area, state);
    }
    if state.new_project_mode {
        render_new_project_overlay(frame, area, state);
    }
    if state.new_series_mode {
        render_new_series_overlay(frame, area, state);
    }
    if state.pick_project_mode {
        render_pick_project_overlay(frame, area, state);
    }
    if state.tag_mode {
        render_tag_overlay(frame, area, state);
    }
    if state.rating_mode {
        render_rating_overlay(frame, area, state);
    }
    if state.confirm_delete_mode {
        render_confirm_delete_overlay(frame, area, state);
    }
    if state.show_metrics_overlay {
        render_author_metrics_overlay(frame, area, state);
    }
    if state.api_key_input_mode {
        render_api_key_input_overlay(frame, area, state);
    }
    if state.custom_field_mode {
        render_custom_field_overlay(frame, area, state);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let online = state.api_mode.allows_api_calls();

    let (ctx_text, ctx_style) = if let Some(name) = &state.active_author {
        (
            format!("👤 {}", name),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )
    } else if let Some(ref notation) = state.active_udc_notation {
        (
            format!("▤ UDC {}", notation),
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        )
    } else if let Some(s) =
        state.series.iter().find(|s| s.id == state.active_series_id)
    {
        (
            format!("≡ {}", s.name),
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )
    } else if let Some(p) = state
        .projects
        .iter()
        .find(|p| p.id == state.active_project_id)
    {
        (
            format!("▣ {}", p.name),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "□ 전체 문헌".to_string(),
            Style::default().fg(Color::Gray),
        )
    };

    let (api_text, api_style) = if online {
        (state.api_mode.as_str().to_string(), Style::default().fg(Color::Cyan))
    } else {
        ("오프라인".to_string(), Style::default().fg(Color::DarkGray))
    };

    let header = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("Libran", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(ctx_text, ctx_style),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(state.document_count.to_string(), Style::default().fg(Color::Yellow)),
        Span::raw(" 문헌  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(api_text, api_style),
    ]))
    .style(Style::default().fg(Color::Gray).bg(Color::Black));

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

    let field_name = crate::app::dispatcher::EDIT_FIELDS
        .get(state.edit_field)
        .unwrap_or(&"?");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" 편집: {} ", field_name),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  필드: ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled(
                format!("{} ({}/{})", field_name, state.edit_field + 1, crate::app::dispatcher::EDIT_FIELDS.len()),
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

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_new_project_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(50, 35, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(" 새 프로젝트 ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

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
            Span::styled("  문헌을 주제별로 묶는 폴더입니다", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
        Line::from(vec![
            Span::styled("  생성 후 m 키로 문헌을 추가하세요", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 생성  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 취소", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_new_series_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(50, 35, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(Span::styled(" 새 시리즈 ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  이름: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Black)),
            Span::styled(state.new_series_input.clone(), Style::default().fg(Color::White).bg(Color::Black)),
            Span::styled("▎", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 생성  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::styled("Esc", Style::default().fg(Color::Cyan).bg(Color::Black)),
            Span::styled(" 취소", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  같은 저널의 여러 호를 묶습니다", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
        Line::from(vec![
            Span::styled("  A 키로 같은 저널 문헌을 자동으로 묶을 수 있습니다", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_pick_project_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(55, 60, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " 프로젝트 선택 ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let search_line = Line::from(vec![
        Span::styled("  검색: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Black)),
        Span::styled(state.pick_project_input.clone(), Style::default().fg(Color::White).bg(Color::Black)),
        Span::styled("▎", Style::default().fg(Color::Cyan)),
    ]);

    let query = state.pick_project_input.to_lowercase();
    let filtered: Vec<&crate::db::projects::Project> = state
        .projects
        .iter()
        .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|p| {
            let count = if let Ok(conn) = state.db.lock() {
                crate::db::projects::count_documents(&conn, p.id.unwrap_or(0)).unwrap_or(0)
            } else {
                0
            };
            let active = state.active_project_id == p.id;
            let icon = if active { "▣" } else { "□" };
            let name_style = if active {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::Black)
            } else {
                Style::default().fg(Color::Gray).bg(Color::Black)
            };
            let icon_style = if active {
                Style::default().fg(Color::Yellow).bg(Color::Black)
            } else {
                Style::default().fg(Color::DarkGray).bg(Color::Black)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", icon), icon_style),
                Span::styled(p.name.clone(), name_style),
                Span::styled(format!(" ({})", count), Style::default().fg(Color::DarkGray).bg(Color::Black)),
            ]))
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    frame.render_widget(Paragraph::new(search_line).style(Style::default().bg(Color::Black)), chunks[0]);

    if items.is_empty() {
        let empty = Paragraph::new(Line::from(vec![
            Span::styled("  일치하는 프로젝트가 없습니다", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        ]))
        .style(Style::default().bg(Color::Black));
        frame.render_widget(empty, chunks[1]);
    } else {
        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶");
        frame.render_stateful_widget(list, chunks[1], &mut ratatui::widgets::ListState::default().with_selected(Some(state.pick_project_cursor)));
    }

    let hint = Line::from(vec![
        Span::styled("  j/k", Style::default().fg(Color::Cyan).bg(Color::Black)),
        Span::styled(" 이동  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::styled("Enter", Style::default().fg(Color::Cyan).bg(Color::Black)),
        Span::styled(" 추가  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::styled("Esc", Style::default().fg(Color::Cyan).bg(Color::Black)),
        Span::styled(" 취소  ", Style::default().fg(Color::DarkGray).bg(Color::Black)),
        Span::styled("문자 입력", Style::default().fg(Color::Cyan).bg(Color::Black)),
        Span::styled(" 검색", Style::default().fg(Color::DarkGray).bg(Color::Black)),
    ]);
    frame.render_widget(Paragraph::new(hint).style(Style::default().bg(Color::Black)), chunks[2]);
}

fn render_tag_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let overlay = Rect {
        x: area.x + 1,
        y: area.bottom().saturating_sub(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, overlay);

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("태그>", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(state.tag_input.clone(), Style::default().fg(Color::White).bg(Color::Black)),
        Span::styled("▎", Style::default().fg(Color::Magenta)),
        Span::raw("  "),
        Span::styled("스페이스 구분  Esc 저장", Style::default().fg(Color::DarkGray).bg(Color::Black)),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, overlay);
}

fn render_rating_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let overlay = Rect {
        x: area.x + 1,
        y: area.bottom().saturating_sub(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };

    frame.render_widget(Clear, overlay);

    let current = state.detail_doc.as_ref().and_then(|d| d.rating);
    let stars = match current {
        Some(r) if (1..=5).contains(&r) => {
            "★".repeat(r as usize) + &"☆".repeat(5 - r as usize)
        }
        _ => "☆".repeat(5),
    };

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("별점>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(stars, Style::default().fg(Color::Yellow).bg(Color::Black)),
        Span::raw("  "),
        Span::styled("1-5 설정  0 삭제  Esc 취소", Style::default().fg(Color::DarkGray).bg(Color::Black)),
    ]);

    let para = Paragraph::new(line).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, overlay);
}

fn render_confirm_delete_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(55, 30, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Span::styled(
            " 문헌 삭제 ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let title_display: String = state
        .delete_confirm_title
        .chars()
        .take(60)
        .collect();
    let title_display = if state.delete_confirm_title.chars().count() > 60 {
        format!("{}…", title_display)
    } else {
        title_display
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  이 문헌을 삭제하시겠습니까?",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  › ", Style::default().fg(Color::Red).bg(Color::Black)),
            Span::styled(
                title_display,
                Style::default().fg(Color::Yellow).bg(Color::Black),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Y/Enter",
                Style::default().fg(Color::Green).bg(Color::Black),
            ),
            Span::styled(" 예   ", Style::default().fg(Color::Gray).bg(Color::Black)),
            Span::styled(
                "N/Esc",
                Style::default().fg(Color::Cyan).bg(Color::Black),
            ),
            Span::styled(
                " 아니오",
                Style::default().fg(Color::Gray).bg(Color::Black),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  S",
                Style::default().fg(Color::Magenta).bg(Color::Black),
            ),
            Span::styled(
                " 앞으로 확인 없이 삭제",
                Style::default().fg(Color::DarkGray).bg(Color::Black),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_author_metrics_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " 연구자 지표 ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let metrics = state.author_metrics.get(&state.metrics_overlay_name);
    let lines = if let Some(m) = metrics {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  이름:  ",
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                ),
                Span::styled(
                    m.name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  h-index:   ",
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                ),
                Span::styled(
                    m.h_index.map_or("—".to_string(), |v| v.to_string()),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  i10-index: ",
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                ),
                Span::styled(
                    m.i10_index.map_or("—".to_string(), |v| v.to_string()),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  논문 수:   ",
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                ),
                Span::styled(
                    m.works_count.map_or("—".to_string(), |v| v.to_string()),
                    Style::default().fg(Color::White).bg(Color::Black),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  총 인용:   ",
                    Style::default().fg(Color::Yellow).bg(Color::Black),
                ),
                Span::styled(
                    m.cited_by_count.map_or("—".to_string(), |v| v.to_string()),
                    Style::default().fg(Color::White).bg(Color::Black),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  출처: ",
                    Style::default().fg(Color::DarkGray).bg(Color::Black),
                ),
                Span::styled(
                    m.source.display_name().to_string(),
                    Style::default().fg(Color::Magenta).bg(Color::Black),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Esc/Enter",
                    Style::default().fg(Color::Cyan).bg(Color::Black),
                ),
                Span::styled(
                    " 닫기",
                    Style::default().fg(Color::DarkGray).bg(Color::Black),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  지표를 불러오는 중...",
                Style::default().fg(Color::Yellow).bg(Color::Black),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Esc 닫기",
                Style::default().fg(Color::Cyan).bg(Color::Black),
            )),
        ]
    };

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
    frame.render_widget(para, inner);
}

fn render_api_key_input_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(60, 35, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(Span::styled(
            " OpenAlex API 키 등록 ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let display_key: String = state
        .api_key_input
        .chars()
        .map(|c| if c.is_whitespace() { c } else { '*' })
        .collect();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  키: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            ),
            Span::styled(
                display_key,
                Style::default().fg(Color::White).bg(Color::Black),
            ),
            Span::styled("▎", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  openalex.org/settings/api에서 무료 키 발급",
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        )]),
        Line::from(vec![Span::styled(
            "  비워두면 Semantic Scholar(키 불필요)로 전환",
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default().fg(Color::Cyan).bg(Color::Black),
            ),
            Span::styled(
                " 등록   ",
                Style::default().fg(Color::DarkGray).bg(Color::Black),
            ),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Cyan).bg(Color::Black),
            ),
            Span::styled(
                " 취소",
                Style::default().fg(Color::DarkGray).bg(Color::Black),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray).bg(Color::Black));
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

fn render_custom_field_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " 추가 필드 ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(Color::Gray).bg(Color::Black));

    let key_line = if state.custom_field_editing_key {
        Line::from(vec![
            Span::styled(" 키   : ", Style::default().fg(Color::Yellow)),
            Span::styled(&state.custom_field_key, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" 키   : ", Style::default().fg(Color::DarkGray)),
            Span::styled(&state.custom_field_key, Style::default().fg(Color::Gray)),
        ])
    };

    let value_line = if !state.custom_field_editing_key {
        Line::from(vec![
            Span::styled(" 값   : ", Style::default().fg(Color::Yellow)),
            Span::styled(&state.custom_field_value, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" 값   : ", Style::default().fg(Color::DarkGray)),
            Span::styled(&state.custom_field_value, Style::default().fg(Color::Gray)),
        ])
    };

    let hint = Line::from(vec![Span::styled(
        " [Tab] 키/값 전환  [Enter] 저장  [Esc] 취소",
        Style::default().fg(Color::DarkGray),
    )]);

    let para = Paragraph::new(vec![Line::from(""), key_line, Line::from(""), value_line, Line::from(""), hint])
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, popup);
}
