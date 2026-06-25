use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
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
        Style::default().bg(theme::search_bg())
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

            let (unread_g, reading_g, read_g) = match state.glyph_set.as_str() {
                "ballot" => ("☐ ", "⊡ ", "☒ "),
                _ => ("○ ", "◐ ", "● "),
            };
            let (marker, marker_color) = match (selected, doc.reading_status.as_deref()) {
                (true, Some("read")) => (read_g, theme::selected()),
                (true, Some("reading")) => (reading_g, theme::selected()),
                (true, _) => (unread_g, theme::selected()),
                (false, Some("read")) => (read_g, theme::dim()),
                (false, Some("reading")) => (reading_g, theme::dim()),
                (false, _) => (unread_g, theme::dim()),
            };

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

            let rating_str = match doc.rating {
                Some(r) if (1..=5).contains(&r) => {
                    format!(" {}", "★".repeat(r as usize) + &"☆".repeat(5 - r as usize))
                }
                _ => String::new(),
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(marker_color)),
                    Span::styled(doc.title.clone(), theme::title_style()),
                    if !rating_str.is_empty() {
                        Span::styled(rating_str, Style::default().fg(theme::selected()))
                    } else {
                        Span::raw("")
                    },
                    if score_str.starts_with(" [기준]") {
                        Span::styled(score_str, Style::default().fg(theme::accent_primary()))
                    } else if score_str.starts_with(" [") {
                        Span::styled(score_str, Style::default().fg(theme::selected()))
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
        .style(Style::default().fg(theme::fg()).bg(theme::bg()))
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
        Style::default().bg(theme::bg()).fg(theme::focus_fg())
    } else {
        Style::default().bg(theme::bg()).fg(theme::fg())
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

    let (unread_g, reading_g, read_g) = match state.glyph_set.as_str() {
        "ballot" => ("☐", "⊡", "☒"),
        _ => ("○", "◐", "●"),
    };
    let status_glyph = match doc.reading_status.as_deref() {
        Some("read") => read_g,
        Some("reading") => reading_g,
        _ => unread_g,
    };

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  제목   ", theme::label_style()),
            Span::styled(status_glyph.to_string(), Style::default().fg(theme::selected())),
            Span::raw(" "),
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
    lines.push(field_line("파일 (p로 열기)", doc.file_path.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));

    let reading_label = match doc.reading_status.as_deref() {
        Some("reading") => "읽는 중",
        Some("read") => "읽음",
        _ => "안 읽음",
    };
    lines.push(field_line("읽음 상태 (u)", reading_label));
    lines.push(Line::from(""));
    lines.push(field_line("출처", doc.source.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));

    let rating_display = match doc.rating {
        Some(r) if (1..=5).contains(&r) => {
            format!("{} ({}점)", "★".repeat(r as usize) + &"☆".repeat(5 - r as usize), r)
        }
        _ => "—".to_string(),
    };
    lines.push(field_line("별점", &rating_display));
    lines.push(Line::from(""));

    if state.current_tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  태그   ", theme::label_style()),
            Span::styled("(없음)", theme::dim_style()),
        ]));
    } else {
        let mut tag_spans = vec![
            Span::styled("  태그   ", theme::label_style()),
        ];
        for (i, tag) in state.current_tags.iter().enumerate() {
            if i > 0 {
                tag_spans.push(Span::raw(" "));
            }
            tag_spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(theme::tag()).bg(theme::bg()),
            ));
        }
        lines.push(Line::from(tag_spans));
    }
    lines.push(Line::from(""));

    if let Some(abs) = &doc.abstract_text {
        lines.push(Line::from(vec![Span::styled("  ─── 초록 ───", theme::dim_style())]));
        lines.push(Line::from(""));
        for line in abs.lines().take(10) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), Style::default().fg(theme::fg())),
            ]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled("  초록 없음", theme::dim_style())]));
    }

    if !state.custom_fields.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("  ─── 추가 필드 ───", theme::dim_style())]));
        lines.push(Line::from(""));
        for (_field_id, key, value) in &state.custom_fields {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:6} ", key), theme::label_style()),
                Span::styled(value.clone(), Style::default().fg(theme::fg())),
            ]));
            lines.push(Line::from(""));
        }
    }

    if !state.current_bookmarks.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  ─── 북마크 (b로 추출) ───",
            theme::dim_style(),
        )]));
        lines.push(Line::from(""));
        for (title, page) in state.current_bookmarks.iter().take(20) {
            let page_str = if *page > 0 { format!(" (p.{})", page) } else { String::new() };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{}{}", title, page_str), Style::default().fg(theme::fg())),
            ]));
        }
    }

    let para = Paragraph::new(lines).style(style).wrap(Wrap { trim: false });
    frame.render_widget(para, info_area);

    render_note_section(frame, note_area, state, focused);
}

fn render_note_section(frame: &mut Frame, area: Rect, state: &AppState, detail_focused: bool) {
    if state.note_mode {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::selected()))
            .title(Span::styled(
                " ✎ 노트 (편집 중) ",
                Style::default().fg(theme::selected()).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().fg(theme::fg()).bg(theme::bg()));

        let hint = Line::from(vec![Span::styled(
            " [Enter] 줄바꿈  [Esc] 저장  [Ctrl+D] 삭제",
            Style::default().fg(theme::dim()),
        )]);

        let mut note_lines: Vec<Line> = state.note_input.lines()
            .map(|l| Line::from(vec![
                Span::raw(" "),
                Span::styled(l.to_string(), Style::default().fg(theme::focus_fg()).bg(theme::bg())),
            ]))
            .collect();
        if note_lines.is_empty() {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(" ", Style::default().fg(theme::focus_fg()).bg(theme::bg())),
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
            .border_style(Style::default().fg(theme::divider()))
            .title(Span::styled(
                title,
                Style::default().fg(theme::fg()),
            ))
            .style(Style::default().fg(theme::fg()).bg(theme::bg()));

        let mut note_lines: Vec<Line> = Vec::new();
        if has_note {
            for line in content.lines().take(4) {
                note_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(line.to_string(), Style::default().fg(theme::fg()).bg(theme::bg())),
                ]));
            }
        } else {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("(노트 없음)", Style::default().fg(theme::dim()).bg(theme::bg())),
            ]));
        }

        let hint_text = if detail_focused { "  [n] 노트 작성/수정" } else { "" };
        note_lines.push(Line::from(""));
        note_lines.push(Line::from(vec![
            Span::styled(hint_text, Style::default().fg(theme::dim())),
        ]));

        let para = Paragraph::new(note_lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    }
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    use unicode_width::UnicodeWidthStr;
    let label_width = label.width();
    let target_width: usize = 12;
    let pad = target_width.saturating_sub(label_width);
    Line::from(vec![
        Span::styled(format!("  {}{} ", label, " ".repeat(pad)), theme::label_style()),
        Span::styled(value.to_string(), Style::default().fg(theme::fg())),
    ])
}
