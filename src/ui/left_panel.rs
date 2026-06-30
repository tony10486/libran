use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};

use crate::app::AppState;
use crate::app::state::PanelFocus;
use crate::ui::theme;
use crate::ui::widget_bar;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    // 사이드바 상단 1줄을 위젯 바에 할당, 나머지를 트리에 할당
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    widget_bar::render(frame, chunks[0], state);

    let tree_area = chunks[1];
    let items = build_tree_items(state);
    let focused = state.active_panel == PanelFocus::Left;

    let highlight = if focused {
        theme::focus_style()
    } else {
        Style::default().bg(theme::surface()).fg(theme::dim())
    };

    let list = List::default()
        .items(items)
        .style(Style::default().fg(theme::fg()).bg(theme::surface()))
        .highlight_style(highlight);

    let mut list_state = ListState::default();
    if state.tree_cursor < count_visible_nodes(state) {
        list_state.select(Some(state.tree_cursor));
    }

    frame.render_stateful_widget(list, tree_area, &mut list_state);
}

fn build_tree_items(state: &AppState) -> Vec<ListItem<'static>> {
    let mut items: Vec<ListItem<'static>> = Vec::new();

    // Projects section
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("프로젝트", theme::dim_style()),
        Span::raw(" "),
        Span::styled(
            "────────────────────",
            Style::default().fg(theme::divider()),
        ),
    ])));

    if state.projects.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("(n 키로 생성)", theme::dim_style()),
        ])));
    } else {
        for proj in &state.projects {
            let count = if let Ok(conn) = state.db.lock() {
                crate::db::projects::count_documents(&conn, proj.id.unwrap_or(0)).unwrap_or(0)
            } else {
                0
            };
            let is_active = state.active_project_id == proj.id;
            let (icon, icon_style, name_style) = if is_active {
                (
                    "▸",
                    Style::default().fg(theme::selected()),
                    Style::default()
                        .fg(theme::fg())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "·",
                    Style::default().fg(theme::dim()),
                    Style::default().fg(theme::fg()),
                )
            };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(icon, icon_style),
                Span::raw(" "),
                Span::styled(proj.name.clone(), name_style),
                Span::styled(format!(" ({})", count), theme::dim_style()),
            ])));
        }
    }

    items.push(ListItem::new(""));

    // Series section (optional, shown when series grouping is enabled)
    if state.series_grouping_enabled {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("시리즈", theme::dim_style()),
            Span::raw(" "),
            Span::styled(
                "────────────────────",
                Style::default().fg(theme::divider()),
            ),
        ])));

        if state.series.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled("(S 키로 생성)", theme::dim_style()),
            ])));
        } else {
            for ser in &state.series {
                let count = if let Ok(conn) = state.db.lock() {
                    crate::db::series::count_documents(&conn, ser.id.unwrap_or(0)).unwrap_or(0)
                } else {
                    0
                };
                let is_active = state.active_series_id == ser.id;
                let (icon, icon_style, name_style) = if is_active {
                    (
                        "▸",
                        Style::default().fg(theme::selected()),
                        Style::default()
                            .fg(theme::fg())
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        "·",
                        Style::default().fg(theme::dim()),
                        Style::default().fg(theme::fg()),
                    )
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(icon, icon_style),
                    Span::raw(" "),
                    Span::styled(ser.name.clone(), name_style),
                    Span::styled(format!(" ({})", count), theme::dim_style()),
                ])));
            }
        }

        items.push(ListItem::new(""));
    }

    if !state.authors.is_empty() {
        let arrow = if state.authors_expanded { "▾" } else { "▸" };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(arrow, Style::default().fg(theme::divider())),
            Span::raw(" "),
            Span::styled("연구자별 보기", theme::dim_style()),
            Span::raw(" "),
            Span::styled("────────", Style::default().fg(theme::divider())),
        ])));

        if state.authors_expanded {
            if state.author_search_mode {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        "검색: ",
                        Style::default()
                            .fg(theme::selected())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        state.author_search_input.clone(),
                        Style::default().fg(theme::focus_fg()),
                    ),
                    Span::styled("▎", Style::default().fg(theme::accent_primary())),
                ])));
            }

            let q = state.author_search_input.to_lowercase();
            let filtered: Vec<&(String, i64)> = state
                .authors
                .iter()
                .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
                .collect();

            if filtered.is_empty() && !state.author_search_input.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        "일치하는 연구자가 없습니다",
                        Style::default().fg(theme::dim()),
                    ),
                ])));
            }

            for (name, count) in filtered {
                let is_active = state.active_author.as_deref() == Some(name.as_str());
                let (icon, icon_style, name_style) = if is_active {
                    (
                        "◆",
                        Style::default().fg(theme::selected()),
                        Style::default()
                            .fg(theme::fg())
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        "·",
                        Style::default().fg(theme::dim()),
                        Style::default().fg(theme::fg()),
                    )
                };
                let mut spans = vec![
                    Span::raw("    "),
                    Span::styled(icon, icon_style),
                    Span::raw(" "),
                    Span::styled(name.clone(), name_style),
                    Span::styled(format!(" ({})", count), theme::dim_style()),
                ];
                if let Some(m) = state.author_metrics.get(name) {
                    if let Some(h) = m.h_index {
                        spans.push(Span::styled(
                            format!(" h={}", h),
                            Style::default().fg(theme::key()),
                        ));
                    }
                }
                items.push(ListItem::new(Line::from(spans)));
            }
        }

        items.push(ListItem::new(""));
    }

    // UDC classification section
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("UDC 분류", theme::dim_style()),
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
            Span::styled(arrow, Style::default().fg(theme::divider())),
            Span::raw(" "),
            Span::styled(format!("{:<3} ", notation), theme::code_style()),
            Span::styled(*label, Style::default().fg(theme::fg())),
            count_span,
        ])));

        if expanded && let Some(children) = UDC_CHILDREN.get(*notation) {
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
                    Span::styled(child_arrow, Style::default().fg(theme::divider())),
                    Span::raw(" "),
                    Span::styled(format!("{:<5} ", child_notation), theme::code_style()),
                    Span::styled(child_label.as_str(), Style::default().fg(theme::fg())),
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
            Span::styled("PhySH", theme::dim_style()),
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
                Span::styled(*label, Style::default().fg(theme::fg())),
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

pub fn count_visible_nodes(state: &AppState) -> usize {
    let mut count = 1; // "프로젝트" header
    count += state.projects.len().max(1);
    count += 1; // spacer after projects
    if state.series_grouping_enabled {
        count += 1; // "시리즈" header
        count += state.series.len().max(1);
        count += 1; // spacer after series
    }
    if !state.authors.is_empty() {
        count += 1; // "연구자별 보기" header
        if state.authors_expanded {
            if state.author_search_mode {
                count += 1; // search input line
            }
            let q = state.author_search_input.to_lowercase();
            let filtered_len = state
                .authors
                .iter()
                .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
                .count();
            if filtered_len == 0 && !state.author_search_input.is_empty() {
                count += 1; // "일치하는 연구자가 없습니다"
            } else {
                count += filtered_len;
            }
        }
        count += 1; // spacer after authors
    }
    count += 1; // "UDC 분류" header
    count += UDC_TOP_LEVEL.len();
    for (notation, _) in UDC_TOP_LEVEL {
        if state.expanded_nodes.contains(*notation)
            && let Some(children) = UDC_CHILDREN.get(*notation)
        {
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
    ("4", "(Vacant)·빈 분류"),
    ("5", "자연과학"),
    ("6", "응용과학"),
    ("7", "예술·레크리에이션"),
    ("8", "언어·언어학"),
    ("9", "역사·지리"),
];

use once_cell::sync::Lazy;
use std::collections::HashMap;

pub(crate) static UDC_CHILDREN: Lazy<HashMap<&'static str, Vec<(String, String)>>> =
    Lazy::new(|| {
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
