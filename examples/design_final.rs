//! Design Final — Refined Merger
//!
//! Dark achromatic base with medium-saturation green/blue/lavender accents.
//! Combines: H's ✓/◐/○ read indicators, G's #tag visibility and
//! title-below metadata layout, E's clean detail fields, plus elegant
//! color-contrast progress bars (━ filled / ─ unfilled) inline with metadata.
//! No box-drawing header legends — sections use muted text + thin rules.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, stdout};

// ── Palette: dark achromatic + accent ─────────────────────────
const BG: Color = Color::Rgb(22, 22, 22); // #161616
const SURFACE: Color = Color::Rgb(28, 28, 28); // #1c1c1c  left panel
const FOCUS_BG: Color = Color::Rgb(38, 38, 38); // #262626  focused row

const FG: Color = Color::Rgb(200, 200, 200); // #c8c8c8
const FG_DIM: Color = Color::Rgb(128, 128, 128); // #808080
const FAINT: Color = Color::Rgb(72, 72, 72); // #484848

const GREEN: Color = Color::Rgb(126, 184, 138); // #7eb88a  read / success
const BLUE: Color = Color::Rgb(107, 155, 210); // #6b9bd2  UDC / codes
const LAVENDER: Color = Color::Rgb(184, 169, 212); // #b8a9d4  tags
const AMBER: Color = Color::Rgb(212, 166, 87); // #d4a657  reading / warning
const CYAN: Color = Color::Rgb(114, 184, 201); // #72b8c9  IDs / links
const ROSE: Color = Color::Rgb(200, 140, 150); // #c88c96  error

const RULE: Color = Color::Rgb(50, 50, 50); // #323232  thin dividers

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Tree,
    Docs,
    Detail,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Search,
    Help,
}

#[derive(Clone, Copy)]
enum ReadStatus {
    Unread,
    Reading,
    Read,
}

struct Doc {
    code: &'static str,
    title: &'static str,
    authors: &'static str,
    year: &'static str,
    id: &'static str,
    key: &'static str,
    status: ReadStatus,
    rating: u8,
    progress: u8,
    tags: &'static [&'static str],
    journal: &'static str,
}

const DOCS: &[Doc] = &[
    Doc {
        code: "517.9",
        title: "Nonisothermal Diffuse Interface Model for Electrical Breakdown",
        authors: "Smith, J. · Chen, L.",
        year: "2024",
        id: "doi:10.33048/SIBJIM.2022.25.103",
        key: "Smith2024",
        status: ReadStatus::Read,
        rating: 5,
        progress: 100,
        tags: &["PDE", "breakdown"],
        journal: "SIBJIM",
    },
    Doc {
        code: "51-72",
        title: "Mathematical Modeling in Physics: A Unified Approach",
        authors: "Kim, D.",
        year: "2023",
        id: "doi:10.1006/jmbi.2023.2354",
        key: "Kim2023",
        status: ReadStatus::Reading,
        rating: 4,
        progress: 62,
        tags: &["modeling"],
        journal: "J. Math. Biol.",
    },
    Doc {
        code: "35-XX",
        title: "PDE Solutions for Nonlinear Systems via Decomposition",
        authors: "Lee, S. · Park, M.",
        year: "2023",
        id: "arXiv:2301.00123",
        key: "Lee2023",
        status: ReadStatus::Unread,
        rating: 0,
        progress: 0,
        tags: &["PDE", "nonlinear"],
        journal: "arXiv",
    },
    Doc {
        code: "538.9",
        title: "Quantum Hall Effect in Graphene: Recent Advances",
        authors: "Zhang, Y. · Wang, H. · Liu, Q.",
        year: "2024",
        id: "doi:10.1103/PhysRevLett.132.046",
        key: "Zhang2024",
        status: ReadStatus::Read,
        rating: 3,
        progress: 100,
        tags: &["graphene", "quantum"],
        journal: "Phys. Rev. Lett.",
    },
    Doc {
        code: "004",
        title: "Neural Network Acceleration on Edge Devices",
        authors: "Tanaka, R. · Singh, A.",
        year: "2024",
        id: "arXiv:2403.05678",
        key: "Tanaka2024",
        status: ReadStatus::Reading,
        rating: 0,
        progress: 35,
        tags: &["ML", "edge"],
        journal: "arXiv",
    },
    Doc {
        code: "512",
        title: "Algebraic Structures in Quantum Computation",
        authors: "Brown, E. · Garcia, M.",
        year: "2022",
        id: "doi:10.1090/jam/2022.045",
        key: "Brown2022",
        status: ReadStatus::Unread,
        rating: 0,
        progress: 0,
        tags: &["algebra", "quantum"],
        journal: "J. Appl. Math.",
    },
];

const PROJECTS: &[(&str, u8, u8, ReadStatus)] = &[
    ("ML 가속 연구", 2, 1, ReadStatus::Reading),
    ("CUI 렌더러 설계", 1, 1, ReadStatus::Read),
    ("수학적 모델링", 3, 2, ReadStatus::Reading),
];

const UDC_TREE: &[(&str, &str, u8, &[(&str, &str, u8)])] = &[
    ("0", "총류·정보학", 1, &[("004", "컴퓨터과학", 1)]),
    ("1", "철학·심리학", 0, &[]),
    (
        "5",
        "자연과학",
        4,
        &[
            ("51", "수학", 2),
            ("517", "해석학", 1),
            ("53", "물리학", 2),
            ("538.9", "응축물질물리학", 1),
        ],
    ),
    ("6", "응용과학", 0, &[]),
    ("9", "역사·지리", 0, &[]),
];

// ── Progress bar helper ───────────────────────────────────────
/// Build a thin progress bar using thick `━` (filled, accent) and
/// thin `─` (unfilled, faint). Color-contrast, not character-contrast.
fn progress_bar(pct: u8, bar_len: usize, filled_color: Color) -> Vec<Span<'static>> {
    let filled = (pct as usize * bar_len / 100).min(bar_len);
    let unfilled = bar_len - filled;
    let bar_filled: String = "━".repeat(filled);
    let bar_unfilled: String = "─".repeat(unfilled);
    vec![
        Span::styled(bar_filled, Style::default().fg(filled_color)),
        Span::styled(bar_unfilled, Style::default().fg(FAINT)),
    ]
}

fn progress_color(pct: u8) -> Color {
    if pct == 100 {
        GREEN
    } else if pct > 0 {
        AMBER
    } else {
        FAINT
    }
}

// ── Main ──────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tree_state = ListState::default();
    tree_state.select(Some(3));
    let mut doc_state = ListState::default();
    doc_state.select(Some(0));

    let mut focus = Focus::Tree;
    let mut mode = Mode::Normal;
    let mut search_input = String::new();
    let mut show_detail = false;
    let expanded_udc: Vec<&str> = vec!["5"];
    let mut selected_docs: Vec<usize> = vec![];

    loop {
        terminal.draw(|f| {
            render(
                f,
                &mut tree_state,
                &mut doc_state,
                &focus,
                &mode,
                &search_input,
                show_detail,
                &expanded_udc,
                &selected_docs,
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match mode {
                Mode::Help => {
                    if matches!(
                        key.code,
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
                    ) {
                        mode = Mode::Normal;
                    }
                    continue;
                }
                Mode::Search => {
                    match key.code {
                        KeyCode::Esc => {
                            mode = Mode::Normal;
                            search_input.clear();
                        }
                        KeyCode::Enter => {
                            mode = Mode::Normal;
                        }
                        KeyCode::Backspace => {
                            search_input.pop();
                        }
                        KeyCode::Char(c) => {
                            search_input.push(c);
                        }
                        _ => {}
                    }
                    continue;
                }
                Mode::Normal => {}
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if show_detail {
                        show_detail = false;
                        focus = Focus::Docs;
                    } else {
                        break;
                    }
                }
                KeyCode::Tab => {
                    focus = match (&focus, show_detail) {
                        (Focus::Tree, _) => Focus::Docs,
                        (Focus::Docs, false) => Focus::Tree,
                        (Focus::Docs, true) => Focus::Detail,
                        (Focus::Detail, _) => Focus::Docs,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => match focus {
                    Focus::Tree => tree_state.select_next(),
                    Focus::Docs if !show_detail => doc_state.select_next(),
                    _ => {}
                },
                KeyCode::Up | KeyCode::Char('k') => match focus {
                    Focus::Tree => tree_state.select_previous(),
                    Focus::Docs if !show_detail => doc_state.select_previous(),
                    _ => {}
                },
                KeyCode::Enter => {
                    if focus == Focus::Docs {
                        show_detail = !show_detail;
                        if show_detail {
                            focus = Focus::Detail;
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if focus == Focus::Docs && !show_detail {
                        if let Some(idx) = doc_state.selected() {
                            if let Some(pos) = selected_docs.iter().position(|&i| i == idx) {
                                selected_docs.remove(pos);
                            } else {
                                selected_docs.push(idx);
                            }
                        }
                    }
                }
                KeyCode::Char('/') => {
                    mode = Mode::Search;
                    search_input.clear();
                }
                KeyCode::Char('?') => {
                    mode = Mode::Help;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

// ── Root render ───────────────────────────────────────────────
fn render(
    f: &mut Frame,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
    show_detail: bool,
    expanded_udc: &[&str],
    selected_docs: &[usize],
) {
    let area = f.area();
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, chunks[0]);

    if show_detail {
        render_detail_view(f, chunks[1], doc_state, *focus, search_input);
    } else {
        render_list_view(
            f,
            chunks[1],
            tree_state,
            doc_state,
            *focus,
            expanded_udc,
            selected_docs,
        );
    }

    render_status_bar(f, chunks[2], focus, selected_docs.len());

    if *mode == Mode::Search {
        render_search_overlay(f, area, search_input);
    }
    if *mode == Mode::Help {
        render_help_overlay(f, area);
    }
}

// ── Header ────────────────────────────────────────────────────
fn render_header(f: &mut Frame, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Row 1: brand + project + status
    let (total, read, reading, unread) = doc_counts();
    let title = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "Libran",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("수학적 모델링", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled(
            format!("{}", total),
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 문헌", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("●", Style::default().fg(GREEN)),
        Span::styled(" 오프라인", Style::default().fg(FG_DIM)),
    ]))
    .style(Style::default().bg(BG));
    f.render_widget(title, rows[0]);

    // Row 2: stats + progress
    let read_pct = if total > 0 {
        read as u8 * 100 / total as u8
    } else {
        0
    };
    let mut bar_spans = progress_bar(read_pct, 24, GREEN);

    let mut spans = vec![
        Span::raw("  "),
        Span::styled(
            format!("{}", read),
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 읽음", Style::default().fg(FG_DIM)),
        Span::raw("   "),
        Span::styled(
            format!("{}", reading),
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 읽는중", Style::default().fg(FG_DIM)),
        Span::raw("   "),
        Span::styled(
            format!("{}", unread),
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 안읽음", Style::default().fg(FG_DIM)),
        Span::raw("  "),
    ];
    spans.append(&mut bar_spans);
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        format!("{}%", read_pct),
        Style::default().fg(GREEN),
    ));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        rows[1],
    );
}

fn doc_counts() -> (usize, usize, usize, usize) {
    let total = DOCS.len();
    let read = DOCS
        .iter()
        .filter(|d| matches!(d.status, ReadStatus::Read))
        .count();
    let reading = DOCS
        .iter()
        .filter(|d| matches!(d.status, ReadStatus::Reading))
        .count();
    let unread = DOCS
        .iter()
        .filter(|d| matches!(d.status, ReadStatus::Unread))
        .count();
    (total, read, reading, unread)
}

// ── List view ─────────────────────────────────────────────────
fn render_list_view(
    f: &mut Frame,
    area: Rect,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: Focus,
    expanded_udc: &[&str],
    selected_docs: &[usize],
) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Min(1)])
        .split(area);

    render_tree(f, body[0], tree_state, focus == Focus::Tree, expanded_udc);
    render_doc_list(f, body[1], doc_state, focus == Focus::Docs, selected_docs);
}

// ── Tree panel ────────────────────────────────────────────────
fn render_tree(
    f: &mut Frame,
    area: Rect,
    state: &mut ListState,
    focused: bool,
    expanded_udc: &[&str],
) {
    let mut items: Vec<ListItem> = Vec::new();

    // Projects — no box header, just a subtle label
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "프로젝트",
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("─".repeat(12), Style::default().fg(RULE)),
    ])));

    for (name, total, done, _status) in PROJECTS {
        let pct = (*done as f64 / *total as f64 * 100.0) as u8;
        let pc = progress_color(pct);
        let mut bar = progress_bar(pct, 8, pc);

        let mut spans = vec![
            Span::raw("  "),
            Span::styled("▸", Style::default().fg(FG_DIM)),
            Span::raw(" "),
            Span::styled(*name, Style::default().fg(FG)),
        ];
        spans.append(&mut bar);
        spans.push(Span::styled(format!(" {}%", pct), Style::default().fg(pc)));

        items.push(ListItem::new(Line::from(spans)));
    }

    items.push(ListItem::new(""));
    items.push(ListItem::new(""));

    // UDC
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "UDC 분류",
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("─".repeat(11), Style::default().fg(RULE)),
    ])));

    for (notation, label, count, children) in UDC_TREE {
        let is_expanded = expanded_udc.contains(notation);
        let arrow = if is_expanded { "▾" } else { "▸" };

        items.push(ListItem::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(arrow, Style::default().fg(FAINT)),
            Span::raw(" "),
            Span::styled(format!("{:<6}", notation), Style::default().fg(BLUE)),
            Span::styled(*label, Style::default().fg(FG)),
            if *count > 0 {
                Span::styled(format!("  {}", count), Style::default().fg(FG_DIM))
            } else {
                Span::raw("")
            },
        ])));

        if is_expanded {
            for (c_notation, c_label, c_count) in children.iter() {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("·", Style::default().fg(FAINT)),
                    Span::raw(" "),
                    Span::styled(format!("{:<6}", c_notation), Style::default().fg(BLUE)),
                    Span::styled(*c_label, Style::default().fg(FG_DIM)),
                    if *c_count > 0 {
                        Span::styled(format!("  {}", c_count), Style::default().fg(FAINT))
                    } else {
                        Span::raw("")
                    },
                ])));
            }
        }
    }

    let highlight = if focused {
        Style::default().bg(FOCUS_BG).fg(FG)
    } else {
        Style::default().bg(SURFACE)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(SURFACE).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("▎");

    f.render_stateful_widget(list, area, state);
}

// ── Document list ─────────────────────────────────────────────
fn render_doc_list(
    f: &mut Frame,
    area: Rect,
    state: &mut ListState,
    focused: bool,
    selected: &[usize],
) {
    let mut items: Vec<ListItem> = Vec::new();

    for (i, doc) in DOCS.iter().enumerate() {
        let is_selected = selected.contains(&i);

        // Status indicator from Design H
        let (status_icon, status_color) = match doc.status {
            ReadStatus::Read => ("✓", GREEN),
            ReadStatus::Reading => ("◐", AMBER),
            ReadStatus::Unread => ("○", FAINT),
        };

        let check = if is_selected { "✓" } else { " " };
        let check_style = if is_selected { BLUE } else { FAINT };

        let rating_str = if doc.rating > 0 {
            format!("  {}", "★".repeat(doc.rating as usize))
        } else {
            String::new()
        };

        // Line 1: status + title + rating
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(check, Style::default().fg(check_style)),
            Span::raw(" "),
            Span::styled(
                status_icon,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
            if !rating_str.is_empty() {
                Span::styled(rating_str, Style::default().fg(AMBER))
            } else {
                Span::raw("")
            },
        ])));

        // Line 2: metadata row — authors · year · code · key · id + progress bar + tags
        // Compact: all on one line using · separators
        let _id_color = if doc.id.starts_with("doi:") {
            FG_DIM
        } else {
            CYAN
        };
        let pc = progress_color(doc.progress);
        let mut bar_spans = progress_bar(doc.progress, 10, pc);

        let mut line2 = vec![
            Span::raw("      "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.code, Style::default().fg(BLUE)),
            Span::raw("  ·  "),
            Span::styled(format!("[{}]", doc.key), Style::default().fg(GREEN)),
            Span::raw("   "),
        ];
        line2.append(&mut bar_spans);
        if doc.progress > 0 {
            line2.push(Span::styled(
                format!(" {}%", doc.progress),
                Style::default().fg(pc),
            ));
        }

        // Tags inline on same line if space allows, else skip (shown in detail)
        if !doc.tags.is_empty() {
            line2.push(Span::raw("  "));
            for tag in doc.tags {
                line2.push(Span::styled(
                    format!(" #{}", tag),
                    Style::default().fg(LAVENDER),
                ));
            }
        }

        items.push(ListItem::new(Line::from(line2)));

        // Thin separator between docs (just one blank line with a subtle rule)
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(4), Style::default().fg(RULE)),
        ])));
    }

    let highlight = if focused {
        Style::default().bg(FOCUS_BG).fg(FG)
    } else {
        Style::default().bg(BG)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(BG).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("▎");

    f.render_stateful_widget(list, area, state);
}

// ── Detail view ───────────────────────────────────────────────
fn render_detail_view(
    f: &mut Frame,
    area: Rect,
    doc_state: &mut ListState,
    focus: Focus,
    _search: &str,
) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Min(1)])
        .split(area);

    // Left: compact doc list
    let mut items: Vec<ListItem> = Vec::new();
    for doc in DOCS {
        let (icon, color) = match doc.status {
            ReadStatus::Read => ("✓", GREEN),
            ReadStatus::Reading => ("◐", AMBER),
            ReadStatus::Unread => ("○", FAINT),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                icon,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(FG_DIM)),
        ])));
        items.push(ListItem::new(""));
    }

    let hl = if focus == Focus::Docs {
        Style::default().bg(FOCUS_BG).fg(FG)
    } else {
        Style::default().bg(SURFACE)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(SURFACE).fg(FG))
        .highlight_style(hl)
        .highlight_symbol("▎");
    f.render_stateful_widget(list, body[0], doc_state);

    render_detail_panel(f, body[1], focus == Focus::Detail);
}

fn render_detail_panel(f: &mut Frame, area: Rect, focused: bool) {
    let doc = &DOCS[2]; // Lee2023 example

    let (status_icon, status_color) = match doc.status {
        ReadStatus::Read => ("✓", GREEN),
        ReadStatus::Reading => ("◐", AMBER),
        ReadStatus::Unread => ("○", FAINT),
    };

    let pc = progress_color(doc.progress);
    let mut bar_spans = progress_bar(doc.progress, 20, pc);

    // Detail follows Design E's clean field_line style
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                status_icon,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(FG_DIM)),
        ]),
        Line::from(""),
    ];

    // Progress bar
    let mut prog_line = vec![Span::raw("  ")];
    prog_line.append(&mut bar_spans);
    if doc.progress > 0 {
        prog_line.push(Span::styled(
            format!(" {}%", doc.progress),
            Style::default().fg(pc),
        ));
    } else {
        prog_line.push(Span::styled(
            " 안읽음".to_string(),
            Style::default().fg(FAINT),
        ));
    }
    lines.push(Line::from(prog_line));
    lines.push(Line::from(""));

    // Thin rule
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─".repeat(30), Style::default().fg(RULE)),
    ]));
    lines.push(Line::from(""));

    // Fields — Design E style: left-aligned label + value
    let fields: [(&str, &str, Color); 8] = [
        ("저자", doc.authors, FG),
        ("저널", doc.journal, FG_DIM),
        ("연도", doc.year, FG_DIM),
        ("DOI", doc.id, CYAN),
        ("키", doc.key, GREEN),
        ("분류", "517.9 미분방정식 (UDC)", BLUE),
        ("파일", "~/.libran/library/Lee2023.pdf", FAINT),
        ("출처", "PDF 자체 추출", FAINT),
    ];

    for (label, value, color) in fields {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{:<6}", label),
                Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(value, Style::default().fg(color)),
        ]));
        lines.push(Line::from(""));
    }

    // Tags
    if !doc.tags.is_empty() {
        let mut tag_spans = vec![
            Span::raw("  "),
            Span::styled(
                "태그",
                Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
        ];
        for tag in doc.tags {
            tag_spans.push(Span::styled(
                format!(" #{}", tag),
                Style::default().fg(LAVENDER),
            ));
        }
        lines.push(Line::from(tag_spans));
        lines.push(Line::from(""));
    }

    // Rating
    let stars = if doc.rating > 0 {
        "★".repeat(doc.rating as usize) + &"☆".repeat(5 - doc.rating as usize)
    } else {
        "☆".repeat(5)
    };
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "별점",
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(stars, Style::default().fg(AMBER)),
    ]));
    lines.push(Line::from(""));

    // Abstract
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─ ─ 초록 ─ ─", Style::default().fg(RULE)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "비선형 편미분방정식의 해법을 다룬 논문으로,",
            Style::default().fg(FG_DIM),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "분해법과 수치 해석을 결합한 접근을 제시한다.",
            Style::default().fg(FG_DIM),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "특히 반응-확산 계의 해 존재성을 증명하며...",
            Style::default().fg(FAINT),
        ),
    ]));

    let style = if focused {
        Style::default().bg(BG).fg(FG)
    } else {
        Style::default().bg(BG).fg(FG_DIM)
    };

    f.render_widget(
        Paragraph::new(lines)
            .style(style)
            .wrap(Wrap { trim: false }),
        area,
    );
}

// ── Status bar ────────────────────────────────────────────────
fn render_status_bar(f: &mut Frame, area: Rect, focus: &Focus, sel_count: usize) {
    let focus_label = match focus {
        Focus::Tree => "트리",
        Focus::Docs => "문헌",
        Focus::Detail => "상세",
    };

    let hints = [
        ("Tab", "패널"),
        ("j/k", "이동"),
        ("/", "검색"),
        ("␣", "선택"),
        ("Enter", "상세"),
        ("?", "도움말"),
        ("q", "종료"),
    ];
    let right_len: usize = hints
        .iter()
        .map(|(k, l)| k.len() + l.chars().count() + 3)
        .sum::<usize>()
        + (hints.len() - 1) * 2;

    let mut left = vec![
        Span::raw(" "),
        Span::styled("●", Style::default().fg(GREEN)),
        Span::raw(" "),
        Span::styled("준비됨", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled(focus_label, Style::default().fg(LAVENDER)),
    ];
    if sel_count > 0 {
        left.push(Span::raw("  "));
        left.push(Span::styled("·", Style::default().fg(FAINT)));
        left.push(Span::raw("  "));
        left.push(Span::styled(
            format!("{} 선택", sel_count),
            Style::default().fg(AMBER),
        ));
    }

    let left_len: usize = left.iter().map(|s| s.width()).sum();
    let gap = area.width as usize;
    let pad = gap.saturating_sub(left_len + right_len + 2);

    let mut spans = left;
    spans.push(Span::raw(" ".repeat(pad)));
    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(*key, Style::default().fg(BLUE)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::default().fg(FG_DIM)));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        area,
    );
}

// ── Search overlay ────────────────────────────────────────────
fn render_search_overlay(f: &mut Frame, area: Rect, input: &str) {
    let overlay = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: 3,
    };
    f.render_widget(Clear, overlay);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(SURFACE)),
        overlay,
    );

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "⌕",
            Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(FG)),
        Span::styled("▎", Style::default().fg(LAVENDER)),
    ]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(SURFACE)),
        overlay,
    );
}

// ── Help overlay ──────────────────────────────────────────────
fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 70, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(SURFACE)),
        popup,
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "단축키",
                Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("─".repeat(18), Style::default().fg(RULE)),
        ]),
        Line::from(""),
        help_line("Tab", "패널 간 포커스 이동"),
        help_line("j / k", "위·아래 탐색"),
        help_line("Enter", "문헌 상세 보기 / 닫기"),
        help_line("/", "검색 모드 진입"),
        help_line("Space", "문헌 다중 선택 토글"),
        help_line("e", "문헌 메타데이터 편집"),
        help_line("x", "BibTeX / CSL JSON 내보내기"),
        help_line("n", "새 프로젝트 생성"),
        help_line("?", "도움말 토글"),
        help_line("q/Esc", "종료 (상세 보기 중일 시 닫기)"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Tip",
                Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "PDF 파일을 터미널로 드래그하여 추가",
                Style::default().fg(FG_DIM),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc / ?", Style::default().fg(FAINT)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(FAINT)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(SURFACE).fg(FG))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<8}", key), Style::default().fg(BLUE)),
        Span::raw(" "),
        Span::styled(desc.to_string(), Style::default().fg(FG_DIM)),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}
