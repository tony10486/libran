use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Wrap};
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

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(marker_color).bg(Color::Black)),
                    Span::styled(doc.title.clone(), theme::title_style()),
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
    frame.render_widget(para, area);
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:6} ", label), theme::label_style()),
        Span::styled(value.to_string(), Style::default().fg(Color::Gray).bg(Color::Black)),
    ])
}
