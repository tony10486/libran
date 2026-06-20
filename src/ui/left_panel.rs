use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

use crate::app::state::PanelFocus;
use crate::app::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items = build_tree_items(state);
    let focused = state.active_panel == PanelFocus::Left;

    let highlight = if focused {
        theme::focus_style()
    } else {
        Style::default().bg(Color::Black).fg(Color::DarkGray)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(Color::Black))
        .highlight_style(highlight);

    let mut list_state = ListState::default();
    if state.tree_cursor < count_visible_nodes(state) {
        list_state.select(Some(state.tree_cursor));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn build_tree_items(state: &AppState) -> Vec<ListItem<'static>> {
    let mut items: Vec<ListItem<'static>> = Vec::new();

    // Projects section
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("프로젝트", theme::header_style()),
    ])));

    if state.projects.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("(프로젝트 없음)", theme::dim_style()),
        ])));
    } else {
        for proj in &state.projects {
            let marker = if state.active_project_id == proj.id {
                "◆ "
            } else {
                "  "
            };
            let name_style = if state.active_project_id == proj.id {
                Style::default().fg(Color::Cyan).bg(Color::Black)
            } else {
                Style::default().fg(Color::Gray).bg(Color::Black)
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(marker, Style::default().fg(Color::Yellow).bg(Color::Black)),
                Span::styled(proj.name.clone(), name_style),
            ])));
        }
    }

    items.push(ListItem::new(""));

    // UDC classification section
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("UDC 분류", theme::header_style()),
    ])));

    let udc_top = UDC_TOP_LEVEL;
    for (notation, label) in udc_top {
        let expanded = state.expanded_nodes.contains(*notation);
        let facet_count = get_facet_count(state, notation);

        let arrow = if expanded { "▾" } else { "▸" };
        let count_span = if let Some(count) = facet_count {
            Span::styled(format!(" ({})", count), theme::dim_style())
        } else {
            Span::raw("")
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(arrow, Style::default().fg(Color::DarkGray).bg(Color::Black)),
            Span::raw(" "),
            Span::styled(format!("{:<3} ", notation), theme::code_style()),
            Span::styled(*label, Style::default().fg(Color::Gray).bg(Color::Black)),
            count_span,
        ])));

        if expanded
            && let Some(children) = UDC_CHILDREN.get(*notation) {
                for (child_notation, child_label) in children.iter() {
                    let child_expanded = state.expanded_nodes.contains(child_notation.as_str());
                    let child_arrow = if child_expanded { "▾" } else { "▸" };
                    let child_count = get_facet_count(state, child_notation);

                    let child_count_span = if let Some(count) = child_count {
                        Span::styled(format!(" ({})", count), theme::dim_style())
                    } else {
                        Span::raw("")
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(child_arrow, Style::default().fg(Color::DarkGray).bg(Color::Black)),
                        Span::raw(" "),
                        Span::styled(format!("{:<5} ", child_notation), theme::code_style()),
                        Span::styled(child_notation.as_str(), Style::default().fg(Color::Gray).bg(Color::Black)),
                        Span::raw(" "),
                        Span::styled(child_label.as_str(), Style::default().fg(Color::Gray).bg(Color::Black)),
                        child_count_span,
                    ])));
                }
            }
    }

    // PhySH section
    let physh_has_docs = PHYSH_TOP
        .iter()
        .any(|(notation, _)| get_facet_count(state, notation).is_some());

    if physh_has_docs {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("PhySH", theme::header_style()),
        ])));

        for (notation, label) in PHYSH_TOP {
            let count = get_facet_count(state, notation);
            let count_span = if let Some(c) = count {
                Span::styled(format!(" ({})", c), theme::dim_style())
            } else {
                Span::raw("")
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  ▸ "),
                Span::styled(format!("{:<8} ", notation), theme::code_style()),
                Span::styled(*label, Style::default().fg(Color::Gray).bg(Color::Black)),
                count_span,
            ])));
        }
    }

    items
}

fn get_facet_count(state: &AppState, notation: &str) -> Option<i64> {
    state
        .facets
        .iter()
        .find(|f| f.notation == notation)
        .map(|f| f.count)
}

fn count_visible_nodes(state: &AppState) -> usize {
    // Rough count for cursor bounds
    let mut count = 2; // header + spacer
    count += state.projects.len().max(1);
    count += 2; // spacer + UDC header
    count += UDC_TOP_LEVEL.len();
    for (notation, _) in UDC_TOP_LEVEL {
        if state.expanded_nodes.contains(*notation)
            && let Some(children) = UDC_CHILDREN.get(*notation) {
                count += children.len();
            }
    }

    let physh_has_docs = PHYSH_TOP
        .iter()
        .any(|(notation, _)| get_facet_count(state, notation).is_some());

    if physh_has_docs {
        count += 2;
        count += PHYSH_TOP.len();
    }
    count
}

const UDC_TOP_LEVEL: &[(&str, &str)] = &[
    ("0", "총류·정보학"),
    ("1", "철학·심리학"),
    ("2", "종교"),
    ("3", "사회과학"),
    ("5", "자연과학"),
    ("6", "응용과학"),
    ("7", "예술·레크리에이션"),
    ("8", "언어·언어학"),
    ("9", "역사·지리"),
];

use std::collections::HashMap;
use once_cell::sync::Lazy;

static UDC_CHILDREN: Lazy<HashMap<&'static str, Vec<(String, String)>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "5",
        vec![
            ("51".to_string(), "수학".to_string()),
            ("52".to_string(), "천문학".to_string()),
            ("53".to_string(), "물리학".to_string()),
            ("54".to_string(), "화학".to_string()),
            ("55".to_string(), "지질학".to_string()),
            ("57".to_string(), "생물학".to_string()),
        ],
    );
    m.insert(
        "51",
        vec![
            ("512".to_string(), "대수학".to_string()),
            ("514".to_string(), "기하학".to_string()),
            ("517".to_string(), "해석학".to_string()),
        ],
    );
    m.insert(
        "53",
        vec![
            ("531".to_string(), "역학".to_string()),
            ("532".to_string(), "유체역학".to_string()),
            ("535".to_string(), "광학".to_string()),
            ("537".to_string(), "전자기학".to_string()),
            ("538.9".to_string(), "응집물질물리학".to_string()),
        ],
    );
    m.insert(
        "0",
        vec![
            ("004".to_string(), "컴퓨터과학".to_string()),
            ("01".to_string(), "서지학".to_string()),
            ("02".to_string(), "도서관학".to_string()),
        ],
    );
    m.insert(
        "6",
        vec![
            ("61".to_string(), "의학·보건".to_string()),
            ("62".to_string(), "공학".to_string()),
            ("63".to_string(), "농업".to_string()),
        ],
    );
    m
});

const PHYSH_TOP: &[(&str, &str)] = &[
    ("Condensed", "응축물질"),
    ("Particles", "입자물리"),
    ("Gravitation", "중력·우주론"),
    ("Quantum", "양자정보"),
];
