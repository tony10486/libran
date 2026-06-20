use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::PanelFocus;
use crate::app::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.active_panel == PanelFocus::Right;

    let highlight = if focused {
        theme::focus_style()
    } else {
        Style::default().bg(Color::Black).fg(Color::DarkGray)
    };

    let is_sort_mode = state.is_similarity_sorted();

    let items: Vec<ListItem> = if state.documents.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("문헌이 없습니다. PDF 파일을 드래그하여 추가하세요.", theme::dim_style()),
        ]))]
    } else {
        state.documents.iter().map(|doc| {
            let selected = state.selected_doc_ids.contains(&doc.id.unwrap_or(0));
            let marker = if selected { "◆ " } else { "  " };
            let marker_color = if selected { Color::Yellow } else { Color::DarkGray };

            let authors = doc.authors.as_deref().unwrap_or("저자 불명");
            let year = doc.pub_year.map(|y| y.to_string()).unwrap_or_else(|| "n.d.".to_string());
            let doi = doc.doi.as_deref().unwrap_or("");
            let key = doc.citation_key.as_deref().unwrap_or("");

            let score_str = if is_sort_mode {
                if let Some(score) = state.similarity_scores.iter()
                    .find(|s| s.document_id == doc.id.unwrap_or(0))
                {
                    if doc.id == state.similarity_ref_doc_id {
                        " [기준]".to_string()
                    } else if score.total_score > 0.0 {
                        format!(" [{:.1}]", score.total_score)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(marker_color).bg(Color::Black)),
                    Span::styled(doc.title.clone(), theme::title_style()),
                    if score_str.starts_with(" [기준]") {
                        Span::styled(score_str, Style::default().fg(Color::Cyan).bg(Color::Black))
                    } else if score_str.starts_with(" [") {
                        Span::styled(score_str, Style::default().fg(Color::Yellow).bg(Color::Black))
                    } else {
                        Span::raw("")
                    },
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(authors.to_string(), theme::meta_style()),
                    Span::raw("  "),
                    Span::styled(year.clone(), theme::meta_style()),
                    Span::raw("  "),
                    Span::styled(doi.to_string(), theme::meta_style()),
                    Span::raw("  "),
                    Span::styled(format!("[{}]", key), theme::key_style()),
                ]),
                Line::from(""),
            ])
        }).collect()
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(Color::Black))
        .highlight_style(highlight);

    let mut list_state = ListState::default();
    if !state.documents.is_empty() && state.list_cursor < state.documents.len() {
        list_state.select(Some(state.list_cursor));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.active_panel == PanelFocus::Detail;
    let style = if focused {
        Style::default().bg(Color::Black).fg(Color::White)
    } else {
        Style::default().bg(Color::Black).fg(Color::Gray)
    };

    let doc = match &state.detail_doc {
        Some(d) => d,
        None => {
            let para = Paragraph::new("  상세 정보 없음").style(style);
            frame.render_widget(para, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(area);

    let info_area = chunks[0];
    let note_area = chunks[1];

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  제목   ", theme::label_style()),
            Span::styled(doc.title.clone(), theme::title_style()),
        ]),
        Line::from(""),
    ];
    lines.push(field_line("저자", doc.authors.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("저널", doc.journal.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("학회", doc.conference.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("연도", &doc.pub_year.map(|y| y.to_string()).unwrap_or_else(|| "—".to_string())));
    lines.push(Line::from(""));
    lines.push(field_line("DOI", doc.doi.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("arXiv", doc.arxiv_id.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("키", doc.citation_key.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("파일", doc.file_path.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("출처", doc.source.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));

    if let Some(abs) = &doc.abstract_text {
        lines.push(Line::from(vec![Span::styled("  ─── 초록 ───", theme::dim_style())]));
        lines.push(Line::from(""));
        for line in abs.lines().take(10) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), Style::default().fg(Color::Gray).bg(Color::Black)),
            ]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled("  초록 없음", theme::dim_style())]));
    }

    let para = Paragraph::new(lines).style(style).wrap(Wrap { trim: false });
    frame.render_widget(para, info_area);

    render_note_section(frame, note_area, state, focused);
}

fn render_note_section(frame: &mut Frame, area: Rect, state: &AppState, detail_focused: bool) {
    let note_bg = Color::Black;
    let note_border = Color::DarkGray;
    let note_fg = Color::Gray;

    if state.note_mode {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                " ✎ 노트 (편집 중) ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(note_bg));

        let hint = Line::from(vec![Span::styled(
            " [Enter] 줄바꿈  [Esc] 저장  [Ctrl+D] 삭제",
            Style::default().fg(Color::DarkGray),
        )]);

        let mut note_lines: Vec<Line> = state.note_input.lines()
            .map(|l| Line::from(vec![
                Span::raw(" "),
                Span::styled(l.to_string(), Style::default().fg(Color::White).bg(note_bg)),
            ]))
            .collect();
        if note_lines.is_empty() {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(" ", Style::default().fg(Color::White).bg(note_bg)),
            ]));
        }
        note_lines.push(Line::from(""));
        note_lines.push(hint);

        let para = Paragraph::new(note_lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    } else {
        let content = state.current_note.as_deref().unwrap_or("");
        let has_note = !content.is_empty();

        let title = if has_note { " 📝 노트 " } else { " 📝 노트 (없음) " };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(note_border))
            .title(Span::styled(
                title,
                Style::default().fg(note_fg),
            ))
            .style(Style::default().bg(note_bg));

        let mut note_lines: Vec<Line> = Vec::new();
        if has_note {
            for line in content.lines().take(4) {
                note_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(line.to_string(), Style::default().fg(note_fg).bg(note_bg)),
                ]));
            }
        } else {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("(노트 없음)", Style::default().fg(Color::DarkGray).bg(note_bg)),
            ]));
        }

        let hint_text = if detail_focused { "  [n] 노트 작성/수정" } else { "" };
        note_lines.push(Line::from(""));
        note_lines.push(Line::from(vec![
            Span::styled(hint_text, Style::default().fg(Color::DarkGray)),
        ]));

        let para = Paragraph::new(note_lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    }
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:6} ", label), theme::label_style()),
        Span::styled(value.to_string(), Style::default().fg(Color::Gray).bg(Color::Black)),
    ])
}
