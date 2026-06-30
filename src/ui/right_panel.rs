use std::str::FromStr;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::AppState;
use crate::app::state::PanelFocus;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.active_panel == PanelFocus::Right;

    let highlight = if focused {
        theme::focus_style()
    } else {
        Style::default().bg(theme::bg())
    };

    let is_sort_mode = state.is_similarity_sorted();

    let docs = if state.queue_view {
        &state.queue
    } else {
        &state.documents
    };

    let header = if state.queue_view {
        " 읽기 큐 (Q: 추가, R: 제거, J/K: 순서 변경, Y: 종료) "
    } else {
        ""
    };

    let items: Vec<ListItem> = if docs.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if state.queue_view {
                    "읽기 큐가 비어 있습니다. Q 키로 문헌을 추가하세요."
                } else {
                    "문헌이 없습니다. PDF 파일을 드래그하여 추가하세요."
                },
                theme::dim_style(),
            ),
        ]))]
    } else {
        docs.iter()
            .map(|doc| {
                let selected = state.selected_doc_ids.contains(&doc.id.unwrap_or(0));

                let (unread_g, reading_g, read_g) = match state.glyph_set.as_str() {
                    "ballot" => ("☐", "⊡", "☒"),
                    _ => ("○", "◐", "✓"),
                };
                let (marker, marker_color) = match (selected, doc.reading_status.as_deref()) {
                    (true, Some("read")) => (read_g, theme::success()),
                    (true, Some("reading")) => (reading_g, theme::warning()),
                    (true, _) => (unread_g, theme::dim()),
                    (false, Some("read")) => (read_g, theme::success()),
                    (false, Some("reading")) => (reading_g, theme::warning()),
                    (false, _) => (unread_g, theme::dim()),
                };

                let authors = doc.authors.as_deref().unwrap_or("저자 불명");
                let year = doc
                    .pub_year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "n.d.".to_string());
                let doi = doc.doi.as_deref().unwrap_or("");
                let key = doc.citation_key.as_deref().unwrap_or("");

                let score_str = if is_sort_mode {
                    if let Some(score) = state
                        .similarity_scores
                        .iter()
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

                let progress_val = doc.reading_progress.unwrap_or(0);
                let bar_color = if progress_val >= 100 {
                    Color::Rgb(74, 144, 226) // 완독 시 파스텔 파랑
                } else if progress_val > 0 {
                    // 1% ~ 99% 선형 보간 (파스텔 빨강 -> 파스텔 초록)
                    let t = (progress_val.min(99).max(1) - 1) as f64 / 98.0;
                    let r = (255.0 + t * (114.0 - 255.0)).round() as u8;
                    let g = (114.0 + t * (220.0 - 114.0)).round() as u8;
                    let b = (114.0 + t * (114.0 - 114.0)).round() as u8;
                    Color::Rgb(r, g, b)
                } else {
                    theme::dim()
                };

                let mut bar_spans = theme::progress_bar_spans(progress_val as u8, 10, bar_color);

                // Line 1: marker + title + rating + score
                let mut line1_spans = vec![
                    Span::styled(
                        format!("{} ", marker),
                        Style::default()
                            .fg(marker_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(doc.title.clone(), theme::title_style()),
                ];
                if !rating_str.is_empty() {
                    line1_spans.push(Span::styled(rating_str, Style::default().fg(theme::warning())));
                }
                if !score_str.is_empty() {
                    if score_str.starts_with(" [기준]") {
                        line1_spans.push(Span::styled(score_str, Style::default().fg(theme::accent_primary())));
                    } else {
                        line1_spans.push(Span::styled(score_str, Style::default().fg(theme::selected())));
                    }
                }
                let line1 = Line::from(line1_spans);

                // Line 2: metadata row — authors · year · doi · [key] + progress bar
                let mut line2_spans = vec![
                    Span::raw("      "),
                    Span::styled(authors.to_string(), theme::meta_style()),
                ];
                line2_spans.push(Span::styled(" · ", Style::default().fg(theme::divider())));
                line2_spans.push(Span::styled(year.clone(), theme::meta_style()));
                
                if !doi.is_empty() {
                    line2_spans.push(Span::styled(" · ", Style::default().fg(theme::divider())));
                    line2_spans.push(Span::styled(doi.to_string(), theme::meta_style()));
                }
                
                if !key.is_empty() {
                    line2_spans.push(Span::styled(" · ", Style::default().fg(theme::divider())));
                    line2_spans.push(Span::styled(format!("[{}]", key), theme::key_style()));
                }

                line2_spans.push(Span::raw("   "));
                line2_spans.append(&mut bar_spans);
                
                if progress_val > 0 {
                    line2_spans.push(Span::styled(
                        format!(" {}%", progress_val),
                        Style::default().fg(bar_color),
                    ));
                }
                let line2 = Line::from(line2_spans);

                // Line 3: Thin separator rule (a subtle line separating items)
                let line3 = Line::from(vec![
                    Span::raw("      "),
                    Span::styled("─".repeat(4), Style::default().fg(theme::divider())),
                ]);

                ListItem::new(vec![line1, line2, line3])
            })
            .collect()
    };

    let list = List::default()
        .items(items)
        .style(Style::default().fg(theme::fg()).bg(theme::bg()))
        .highlight_style(highlight);

    let mut list_state = ListState::default();
    if !docs.is_empty() && state.list_cursor < docs.len() {
        list_state.select(Some(state.list_cursor));
    }

    if state.queue_view && !header.is_empty() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                header,
                Style::default()
                    .fg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
            )]))
            .style(Style::default().bg(theme::bg())),
            chunks[0],
        );
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
    } else {
        frame.render_stateful_widget(list, area, &mut list_state);
    }
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
        _ => ("○", "◐", "✓"),
    };
    let (status_glyph, status_color) = match doc.reading_status.as_deref() {
        Some("read") => (read_g, theme::success()),
        Some("reading") => (reading_g, theme::warning()),
        _ => (unread_g, theme::dim()),
    };

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  제목   ", theme::label_style()),
            Span::styled(
                format!("{} ", status_glyph),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
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
    lines.push(field_line(
        "연도",
        &doc.pub_year
            .map(|y| y.to_string())
            .unwrap_or_else(|| "—".to_string()),
    ));
    lines.push(Line::from(""));
    lines.push(field_line("DOI", doc.doi.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("arXiv", doc.arxiv_id.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line("키", doc.citation_key.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));
    lines.push(field_line(
        "파일 (p로 열기)",
        doc.file_path.as_deref().unwrap_or("—"),
    ));
    lines.push(Line::from(""));

    if !state.current_attachments.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  첨부   ", theme::label_style()),
            Span::styled(
                format!("{}개", state.current_attachments.len()),
                Style::default().fg(theme::fg()),
            ),
        ]));
        for att in &state.current_attachments {
            let type_label = match att.attachment_type.as_str() {
                "supplementary" => "보충",
                "dataset" => "데이터",
                _ => "기타",
            };
            let label_part = att
                .label
                .as_deref()
                .filter(|l| !l.is_empty())
                .map(|l| format!(" — {}", l))
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("[{}] {}{}", type_label, att.file_path, label_part),
                    Style::default().fg(theme::dim()),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    let reading_label = match doc.reading_status.as_deref() {
        Some("reading") => "읽는 중",
        Some("read") => "읽음",
        _ => "안 읽음",
    };
    lines.push(field_line("읽음 상태 (u)", reading_label));
    lines.push(Line::from(""));
    let progress_val = doc.reading_progress.unwrap_or(0);
    lines.push(field_line("진행률 (>/<)", &format!("{}%", progress_val)));
    lines.push(Line::from(""));
    lines.push(field_line("출처", doc.source.as_deref().unwrap_or("—")));
    lines.push(Line::from(""));

    let rating_display = match doc.rating {
        Some(r) if (1..=5).contains(&r) => {
            format!(
                "{} ({}점)",
                "★".repeat(r as usize) + &"☆".repeat(5 - r as usize),
                r
            )
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
        let mut tag_spans = vec![Span::styled("  태그   ", theme::label_style())];
        for (i, tag) in state.current_tags.iter().enumerate() {
            if i > 0 {
                tag_spans.push(Span::raw(" "));
            }
            let color = state
                .tag_colors
                .get(tag)
                .and_then(|hex| Color::from_str(hex).ok())
                .unwrap_or(theme::tag());
            tag_spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(color).bg(theme::bg()),
            ));
        }
        lines.push(Line::from(tag_spans));
    }
    lines.push(Line::from(""));

    if let Some(abs) = &doc.abstract_text {
        lines.push(Line::from(vec![Span::styled(
            "  ─ ─ 초록 ─ ─",
            theme::divider_style(),
        )]));
        lines.push(Line::from(""));
        for line in abs.lines().take(10) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), Style::default().fg(theme::fg())),
            ]));
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  초록 없음",
            theme::dim_style(),
        )]));
    }

    if !state.custom_fields.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  ─ ─ 추가 필드 ─ ─",
            theme::divider_style(),
        )]));
        lines.push(Line::from(""));
        for (_field_id, key, value) in &state.custom_fields {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}  ", key), theme::label_style()),
                Span::styled(value.clone(), Style::default().fg(theme::fg())),
            ]));
            lines.push(Line::from(""));
        }
    }

    if !state.current_bookmarks.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  ─ ─ 북마크 (b로 추출) ─ ─",
            theme::divider_style(),
        )]));
        lines.push(Line::from(""));
        for (title, page) in state.current_bookmarks.iter().take(20) {
            let page_str = if *page > 0 {
                format!(" (p.{})", page)
            } else {
                String::new()
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{}{}", title, page_str),
                    Style::default().fg(theme::fg()),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .style(style)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, info_area);

    render_note_section(frame, note_area, state, focused);
}

fn render_note_section(frame: &mut Frame, area: Rect, state: &AppState, detail_focused: bool) {
    if state.note_mode {
        let block = theme::create_theme_block("✎ 노트 (편집 중)");

        let hint = Line::from(vec![Span::styled(
            " [Enter] 줄바꿈  [Esc] 저장  [Ctrl+D] 삭제",
            Style::default().fg(theme::dim()),
        )]);

        let mut note_lines: Vec<Line> = state
            .note_input
            .lines()
            .map(|l| {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        l.to_string(),
                        Style::default().fg(theme::focus_fg()).bg(theme::bg()),
                    ),
                ])
            })
            .collect();
        if note_lines.is_empty() {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(" ", Style::default().fg(theme::focus_fg()).bg(theme::bg())),
            ]));
        }
        note_lines.push(Line::from(""));
        note_lines.push(hint);

        let para = Paragraph::new(note_lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    } else {
        let notes = &state.current_notes;
        let has_notes = !notes.is_empty();
        let count = notes.len();

        let title = if has_notes {
            if count > 1 {
                format!(" 📝 노트 ({}개) ", count)
            } else {
                " 📝 노트 ".to_string()
            }
        } else {
            " 📝 노트 (없음) ".to_string()
        };
        let block = theme::create_theme_block(&title);

        let mut note_lines: Vec<Line> = Vec::new();
        if has_notes {
            let latest = &notes[0];
            for line in latest.content.lines().take(4) {
                note_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        line.to_string(),
                        Style::default().fg(theme::fg()).bg(theme::bg()),
                    ),
                ]));
            }
            if count > 1 {
                note_lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        format!("… 외 {}개 노트", count - 1),
                        Style::default().fg(theme::dim()).bg(theme::bg()),
                    ),
                ]));
            }
        } else {
            note_lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "(노트 없음)",
                    Style::default().fg(theme::dim()).bg(theme::bg()),
                ),
            ]));
        }

        let hint_text = if detail_focused {
            "  [n] 노트 작성  [:note] $EDITOR로 작성"
        } else {
            ""
        };
        note_lines.push(Line::from(""));
        note_lines.push(Line::from(vec![Span::styled(
            hint_text,
            Style::default().fg(theme::dim()),
        )]));

        let para = Paragraph::new(note_lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    }
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    use unicode_width::UnicodeWidthStr;
    let label_width = label.width();
    let target_width: usize = 6;
    let pad = target_width.saturating_sub(label_width);
    Line::from(vec![
        Span::styled(
            format!("  {}{}  ", label, " ".repeat(pad)),
            theme::label_style(),
        ),
        Span::styled(value.to_string(), Style::default().fg(theme::fg())),
    ])
}
