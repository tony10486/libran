use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Table, Row as TableRow, Cell};
use ratatui::Frame;

use crate::app::AppState;
use crate::citation::graph::RenderMode;
use crate::ui::theme;
use crate::citation::MatchStatus;
use petgraph::visit::EdgeRef;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(ref gs) = state.graph_state else {
        render_empty(frame, area);
        return;
    };

    match gs.render_mode {
        RenderMode::Visual => render_visual(frame, area, state, gs),
        RenderMode::Table => render_table(frame, area, state, gs),
    }
}

fn render_empty(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::dim()))
        .title(" 인용 그래프 ")
        .style(Style::default().fg(theme::fg()).bg(theme::bg()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let msg = Paragraph::new("g 키로 그래프 생성")
        .style(Style::default().fg(theme::dim()).bg(theme::bg()));
    frame.render_widget(msg, inner);
}

fn render_visual(frame: &mut Frame, area: Rect, _state: &AppState, gs: &crate::app::graph_state::GraphState) {
    let cache_tag = if gs.cache_hit { " [캐시]" } else { "" };
    let node_count = gs.graph.node_count();
    let edge_count = gs.graph.inner.edge_count();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_primary()))
        .title(Span::styled(
            format!(" 인용 그래프 ({} 노드, {} 에지){} ", node_count, edge_count, cache_tag),
            Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(theme::fg()).bg(theme::bg()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    let layout = crate::citation::graph::compute_layout(&gs.graph);

    if layout.node_positions.is_empty() {
        lines.push(Line::from(Span::styled(
            "  그래프에 노드가 없습니다",
            Style::default().fg(theme::dim()),
        )));
    } else {
        let max_row = layout.node_positions.iter().map(|p| p.row).max().unwrap_or(0) as usize;

        let mut row_content: std::collections::HashMap<usize, Vec<Span>> = std::collections::HashMap::new();

        for nl in &layout.node_positions {
            let node_idx = petgraph::graph::NodeIndex::new(nl.node_idx);
            let node = match gs.graph.inner.node_weight(node_idx) {
                Some(n) => n,
                None => continue,
            };

            let is_focused = gs.focused_node == Some(nl.node_idx);
            let is_high_cite = node.citation_count >= 3;

            let label_style = if is_focused {
                Style::default().fg(theme::selected()).add_modifier(Modifier::BOLD)
            } else if is_high_cite {
                Style::default().fg(theme::title_fg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::fg())
            };

            let label_text = if node.citation_count >= 3 {
                format!("**{}**", truncate_label(&node.label, 15))
            } else {
                truncate_label(&node.label, 15)
            };

            let indent = (nl.col as usize).min(inner.width as usize);
            let padding = if indent > 0 { " ".repeat(indent) } else { String::new() };

            let spans = row_content.entry(nl.row as usize).or_default();
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::raw(padding));
            spans.push(Span::styled(label_text, label_style));

            if node.citation_count > 0 && !is_high_cite {
                spans.push(Span::styled(
                    format!("({})", node.citation_count),
                    Style::default().fg(theme::dim()),
                ));
            }
        }

        for row_idx in 0..=max_row {
            if let Some(spans) = row_content.get(&row_idx) {
                lines.push(Line::from(spans.clone()));
            } else {
                lines.push(Line::from(""));
            }
        }

        lines.push(Line::from(""));

        let focused_info = gs.focused_node.and_then(|idx| {
            let nidx = petgraph::graph::NodeIndex::new(idx);
            gs.graph.inner.node_weight(nidx).map(|n| (n.doc_id, n.label.clone(), n.citation_count))
        });

        if let Some((doc_id, label, cite_count)) = focused_info {
            let label_owned = label.clone();
            lines.push(Line::from(vec![
                Span::styled("  포커스: ", Style::default().fg(theme::dim())),
                Span::styled(label_owned, Style::default().fg(theme::selected())),
                Span::styled(format!(" (id:{}, 인용:{}건)", doc_id, cite_count), Style::default().fg(theme::dim())),
            ]));
        }
    }

    let help = Line::from(vec![
        Span::styled(" [g]", Style::default().fg(theme::accent_primary())),
        Span::styled("생성 ", Style::default().fg(theme::dim())),
        Span::styled("[G]", Style::default().fg(theme::accent_primary())),
        Span::styled("새로고침 ", Style::default().fg(theme::dim())),
        Span::styled("[Tab]", Style::default().fg(theme::accent_primary())),
        Span::styled("표모드 ", Style::default().fg(theme::dim())),
        Span::styled("[h/j/k/l]", Style::default().fg(theme::accent_primary())),
        Span::styled("이동 ", Style::default().fg(theme::dim())),
        Span::styled("[Esc]", Style::default().fg(theme::accent_primary())),
        Span::styled("돌아가기", Style::default().fg(theme::dim())),
    ]);
    lines.push(help);

    let para = Paragraph::new(lines).style(Style::default().fg(theme::fg()).bg(theme::bg()));
    frame.render_widget(para, inner);
}

fn render_table(frame: &mut Frame, area: Rect, _state: &AppState, gs: &crate::app::graph_state::GraphState) {
    let cache_tag = if gs.cache_hit { " [캐시]" } else { "" };
    let node_count = gs.graph.node_count();
    let edge_count = gs.graph.inner.edge_count();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_primary()))
        .title(Span::styled(
            format!(" 인용 그래프 표 ({}, {}){} ", node_count, edge_count, cache_tag),
            Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().fg(theme::fg()).bg(theme::bg()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = TableRow::new(vec![
        Cell::from("인용 문헌").style(Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD)),
        Cell::from("피인용 문헌").style(Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD)),
        Cell::from("매치").style(Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD)),
        Cell::from("신뢰도").style(Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD)),
    ]);

    let mut rows: Vec<TableRow> = Vec::new();
    let mut edge_idx = 0usize;

    for edge_ref in gs.graph.inner.edge_references() {
        let source = match gs.graph.inner.node_weight(edge_ref.source()) {
            Some(n) => n,
            None => continue,
        };
        let target = match gs.graph.inner.node_weight(edge_ref.target()) {
            Some(n) => n,
            None => continue,
        };

        let is_focused = gs.focused_node == Some(edge_ref.source().index())
            || gs.focused_node == Some(edge_ref.target().index());

        let row_style = if is_focused {
            Style::default().fg(theme::selected()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::fg())
        };

        let status_label = match_status_label(&edge_ref.weight().match_status);

        rows.push(TableRow::new(vec![
            Cell::from(truncate_label(&source.label, 20)).style(row_style),
            Cell::from(truncate_label(&target.label, 20)).style(row_style),
            Cell::from(status_label).style(row_style),
            Cell::from(format!("{:.2}", edge_ref.weight().confidence)).style(row_style),
        ]));

        edge_idx += 1;
        if edge_idx >= inner.height as usize / 2 {
            break;
        }
    }

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Length(20), Constraint::Length(10), Constraint::Length(8)])
        .header(header)
        .style(Style::default().fg(theme::fg()).bg(theme::bg()))
        .block(Block::default().style(Style::default().fg(theme::fg()).bg(theme::bg())));

    frame.render_widget(table, inner);
}

fn match_status_label(status: &MatchStatus) -> &'static str {
    match status {
        MatchStatus::AutoDoi => "DOI",
        MatchStatus::AutoArxiv => "arXiv",
        MatchStatus::AutoTitle => "제목",
        MatchStatus::AutoFuzzy => "퍼지",
        MatchStatus::Manual => "⊕수동",
        MatchStatus::BibtexImport => "⊕BibTeX",
    }
}

fn truncate_label(label: &str, max_len: usize) -> String {
    if label.len() <= max_len {
        label.to_string()
    } else {
        let truncated: String = label.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}
