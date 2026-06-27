//! Design H — "Neo-Grid" Brutalist Terminal
//!
//! Inspired by btop, gh-dash, and brutalist terminal UIs. High-contrast
//! neon accents on pure black, with bold box-drawing borders (━┃┏┓┗┛),
//! filled header bars, and ASCII-art progress bars (▓░). Info-dense
//! and grid-structured — every pixel earns its place.

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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, stdout};

// ── Neo-Grid palette ──────────────────────────────────────────
const BG: Color = Color::Rgb(13, 13, 13); // #0d0d0d — pure dark
const BG_PANEL: Color = Color::Rgb(18, 18, 18); // #121212 — panel bg
const BG_HEADER: Color = Color::Rgb(0, 255, 156); // neon green header bar
const FG: Color = Color::Rgb(230, 230, 230); // #e6e6e6 — bright text
const FG_DIM: Color = Color::Rgb(120, 120, 120); // #787878 — dim text
const NEON: Color = Color::Rgb(0, 255, 156); // #00ff9c — neon green
const GREEN: Color = Color::Rgb(0, 200, 120); // #00c878 — read status green
const MAGENTA: Color = Color::Rgb(255, 92, 138); // #ff5c8a — hot magenta
const YELLOW: Color = Color::Rgb(255, 204, 0); // #ffcc00 — electric yellow
const BLUE: Color = Color::Rgb(94, 168, 255); // #5ea8ff — cyber blue
const CYAN: Color = Color::Rgb(0, 220, 220); // #00dcdc — cyan
const RED: Color = Color::Rgb(255, 85, 85); // #ff5555 — alert red
const FOCUS_BG: Color = Color::Rgb(28, 28, 28); // #1c1c1c — focus row
const BORDER: Color = Color::Rgb(50, 50, 50); // #323232 — grid lines

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

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tree_state = ListState::default();
    tree_state.select(Some(4));
    let mut doc_state = ListState::default();
    doc_state.select(Some(0));

    let mut focus = Focus::Tree;
    let mut mode = Mode::Normal;
    let mut search_input = String::new();
    let mut show_detail = false;
    let expanded_udc: Vec<&str> = vec!["5"];

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

fn render(
    f: &mut Frame,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
    show_detail: bool,
    expanded_udc: &[&str],
) {
    let area = f.area();
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), area);

    // Draw outer border frame
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_header(f, chunks[0]);

    if show_detail {
        render_detail_view(f, chunks[1], doc_state, *focus);
    } else {
        render_list_view(f, chunks[1], tree_state, doc_state, *focus, expanded_udc);
    }

    render_status_bar(f, chunks[2], focus, show_detail);

    if *mode == Mode::Search {
        render_search_overlay(f, area, search_input);
    }
    if *mode == Mode::Help {
        render_help_overlay(f, area);
    }
}

// ── Header — neon filled bar ──────────────────────────────────
fn render_header(f: &mut Frame, area: Rect) {
    // Filled neon header bar
    let header_bar = Rect { height: 1, ..area };
    let title = format!(" LIBRAN ▐ BIBLIOGRAPHY MANAGER ▐ 수학적 모델링 ");
    let padded = format!("{:<width$}", title, width = area.width as usize);
    f.render_widget(
        Paragraph::new(Span::styled(
            padded,
            Style::default()
                .fg(BG)
                .bg(NEON)
                .add_modifier(Modifier::BOLD),
        )),
        header_bar,
    );

    // Stats row
    let stats_area = Rect {
        y: area.y + 1,
        height: 2,
        ..area
    };

    let (total, read, reading, unread) = (
        DOCS.len(),
        DOCS.iter()
            .filter(|d| matches!(d.status, ReadStatus::Read))
            .count(),
        DOCS.iter()
            .filter(|d| matches!(d.status, ReadStatus::Reading))
            .count(),
        DOCS.iter()
            .filter(|d| matches!(d.status, ReadStatus::Unread))
            .count(),
    );

    let read_pct = (read as f64 / total as f64 * 100.0) as u8;
    let bar_len = 24usize;
    let filled = (read_pct as usize * bar_len / 100).min(bar_len);
    let bar: String = "▓".to_string().repeat(filled) + &"░".to_string().repeat(bar_len - filled);

    let stats = Line::from(vec![
        Span::raw(" "),
        Span::styled("┃", Style::default().fg(NEON)),
        Span::styled(
            " TOTAL ",
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:<3}", total), Style::default().fg(FG)),
        Span::raw(" "),
        Span::styled("┃", Style::default().fg(GREEN)),
        Span::styled(
            " READ ",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:<3}", read), Style::default().fg(GREEN)),
        Span::raw(" "),
        Span::styled("┃", Style::default().fg(YELLOW)),
        Span::styled(
            " READING ",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:<3}", reading), Style::default().fg(YELLOW)),
        Span::raw(" "),
        Span::styled("┃", Style::default().fg(FG_DIM)),
        Span::styled(
            " UNREAD ",
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:<3}", unread), Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("┃", Style::default().fg(BORDER)),
        Span::raw("  "),
        Span::styled(bar, Style::default().fg(NEON)),
        Span::raw(" "),
        Span::styled(
            format!("{:>3}%", read_pct),
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(stats).style(Style::default().bg(BG)),
        stats_area,
    );
}

// ── List view ─────────────────────────────────────────────────
fn render_list_view(
    f: &mut Frame,
    area: Rect,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: Focus,
    expanded_udc: &[&str],
) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

    render_tree(f, body[0], tree_state, focus == Focus::Tree, expanded_udc);

    // Vertical divider
    let divider = Paragraph::new("").style(Style::default().bg(BG)).block(
        Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(BORDER)),
    );
    f.render_widget(divider, body[1]);

    render_doc_list(f, body[2], doc_state, focus == Focus::Docs);
}

fn render_tree(
    f: &mut Frame,
    area: Rect,
    state: &mut ListState,
    focused: bool,
    expanded_udc: &[&str],
) {
    let mut items: Vec<ListItem> = Vec::new();

    // Projects with ASCII progress bars
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┏━━━━━━━━━━━━━━━━━━━━┓",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            "┃ ",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
        Span::styled(
            "PROJECTS",
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ┃",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
    ])));
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┗━━━━━━━━━━━━━━━━━━━━┛",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(""));

    for (name, total, done, _status) in PROJECTS {
        let pct = (*done as f64 / *total as f64 * 100.0) as u8;
        let bar_filled = (pct as usize * 10 / 100).min(10);
        let bar = "▓".repeat(bar_filled) + &"░".to_string().repeat(10 - bar_filled);
        let bar_color = if pct == 100 {
            NEON
        } else if pct > 50 {
            YELLOW
        } else {
            MAGENTA
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled("▶ ", Style::default().fg(NEON)),
            Span::styled(*name, Style::default().fg(FG)),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(bar, Style::default().fg(bar_color)),
            Span::raw(" "),
            Span::styled(
                format!("{}/{} {}%", done, total, pct),
                Style::default().fg(FG_DIM),
            ),
        ])));
        items.push(ListItem::new(""));
    }

    // UDC
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┏━━━━━━━━━━━━━━━━━━━━┓",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            "┃ ",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
        Span::styled(
            "UDC 분류",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ┃",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
    ])));
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┗━━━━━━━━━━━━━━━━━━━━┛",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(""));

    for (notation, label, count, children) in UDC_TREE {
        let is_expanded = expanded_udc.contains(notation);
        let arrow = if is_expanded { "▼" } else { "▶" };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(arrow, Style::default().fg(NEON)),
            Span::raw(" "),
            Span::styled(
                format!("{:<6}", notation),
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(*label, Style::default().fg(FG)),
            if *count > 0 {
                Span::styled(format!(" [{}]", count), Style::default().fg(YELLOW))
            } else {
                Span::raw("")
            },
        ])));

        if is_expanded {
            for (c_notation, c_label, c_count) in children.iter() {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("├", Style::default().fg(BORDER)),
                    Span::raw(" "),
                    Span::styled(format!("{:<6}", c_notation), Style::default().fg(CYAN)),
                    Span::styled(*c_label, Style::default().fg(FG_DIM)),
                    if *c_count > 0 {
                        Span::styled(format!(" [{}]", c_count), Style::default().fg(FG_DIM))
                    } else {
                        Span::raw("")
                    },
                ])));
            }
        }
    }

    let highlight = if focused {
        Style::default()
            .bg(FOCUS_BG)
            .fg(FG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(BG_PANEL)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(BG_PANEL).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("▶");

    f.render_stateful_widget(list, area, state);
}

// ── Document list ─────────────────────────────────────────────
fn render_doc_list(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let mut items: Vec<ListItem> = Vec::new();

    // Column header bar
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            "┃ ",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
        Span::styled("ST", Style::default().fg(NEON).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            "UDC   ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "TITLE",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "AUTHORS",
            Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "YEAR",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "PROGRESS",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ┃",
            Style::default().fg(if focused { NEON } else { BORDER }),
        ),
    ])));
    items.push(ListItem::new(Line::from(vec![Span::styled(
        "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )])));
    items.push(ListItem::new(""));

    for doc in DOCS {
        let (status_icon, status_color) = match doc.status {
            ReadStatus::Read => ("✓", NEON),
            ReadStatus::Reading => ("◐", YELLOW),
            ReadStatus::Unread => ("○", FG_DIM),
        };

        let bar_filled = (doc.progress as usize * 10 / 100).min(10);
        let progress_bar = "▓".repeat(bar_filled) + &"░".to_string().repeat(10 - bar_filled);
        let prog_color = if doc.progress == 100 {
            NEON
        } else if doc.progress > 0 {
            YELLOW
        } else {
            FG_DIM
        };

        let id_color = if doc.id.starts_with("doi:") {
            FG_DIM
        } else {
            CYAN
        };
        let rating_str = if doc.rating > 0 {
            format!(" {}", "★".repeat(doc.rating as usize))
        } else {
            String::new()
        };

        // Line 1: status + code + title + rating
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", status_icon),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<6}", doc.code), Style::default().fg(CYAN)),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
            if !rating_str.is_empty() {
                Span::styled(rating_str, Style::default().fg(YELLOW))
            } else {
                Span::raw("")
            },
        ])));

        // Line 2: authors + year + id + key
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  "),
            Span::styled(doc.year, Style::default().fg(YELLOW)),
            Span::raw("  "),
            Span::styled(doc.id, Style::default().fg(id_color)),
            Span::raw("  "),
            Span::styled(format!("[{}]", doc.key), Style::default().fg(NEON)),
        ])));

        // Line 3: progress bar
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(progress_bar, Style::default().fg(prog_color)),
            Span::raw(" "),
            Span::styled(
                format!("{:>3}%", doc.progress),
                Style::default().fg(prog_color).add_modifier(Modifier::BOLD),
            ),
        ])));

        items.push(ListItem::new(""));
    }

    let highlight = if focused {
        Style::default()
            .bg(FOCUS_BG)
            .fg(FG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(BG)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(BG).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("▶");

    f.render_stateful_widget(list, area, state);
}

// ── Detail view ───────────────────────────────────────────────
fn render_detail_view(f: &mut Frame, area: Rect, doc_state: &mut ListState, focus: Focus) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(area);

    // Compact list
    let mut items: Vec<ListItem> = Vec::new();
    for doc in DOCS {
        let (icon, color) = match doc.status {
            ReadStatus::Read => ("✓", NEON),
            ReadStatus::Reading => ("◐", YELLOW),
            ReadStatus::Unread => ("○", FG_DIM),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  "),
            Span::styled(doc.year, Style::default().fg(FG_DIM)),
        ])));
        items.push(ListItem::new(""));
    }

    let highlight = if focus == Focus::Docs {
        Style::default()
            .bg(FOCUS_BG)
            .fg(FG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(BG_PANEL)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(BG_PANEL).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("▶");
    f.render_stateful_widget(list, body[0], doc_state);

    // Divider
    let divider = Paragraph::new("").style(Style::default().bg(BG)).block(
        Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(BORDER)),
    );
    f.render_widget(divider, body[1]);

    render_detail_panel(f, body[2], focus == Focus::Detail);
}

fn render_detail_panel(f: &mut Frame, area: Rect, focused: bool) {
    let doc = &DOCS[2];

    let (status_icon, status_color) = match doc.status {
        ReadStatus::Read => ("✓", NEON),
        ReadStatus::Reading => ("◐", YELLOW),
        ReadStatus::Unread => ("○", FG_DIM),
    };

    let bar_filled = (doc.progress as usize * 20 / 100).min(20);
    let progress_bar = "▓".repeat(bar_filled) + &"░".to_string().repeat(20 - bar_filled);

    let mut lines = vec![
        Line::from(""),
        // Bordered title
        Line::from(vec![Span::styled(
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            Style::default().fg(if focused { NEON } else { BORDER }),
        )]),
        Line::from(vec![
            Span::styled(
                "┃ ",
                Style::default().fg(if focused { NEON } else { BORDER }),
            ),
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
            Span::styled(
                "┃ ",
                Style::default().fg(if focused { NEON } else { BORDER }),
            ),
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  "),
            Span::styled(doc.year, Style::default().fg(YELLOW)),
        ]),
        Line::from(vec![Span::styled(
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            Style::default().fg(if focused { NEON } else { BORDER }),
        )]),
        Line::from(""),
        // Progress bar
        Line::from(vec![
            Span::styled(
                "┃ PROGRESS ┃ ",
                Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
            ),
            Span::styled(progress_bar, Style::default().fg(YELLOW)),
            Span::raw(" "),
            Span::styled(
                format!("{}%", doc.progress),
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Fields with bracket labels
    let fields = [
        ("저자", doc.authors, FG),
        ("저널", "SIAM J. Math. Anal.", FG_DIM),
        ("연도", doc.year, YELLOW),
        ("DOI", doc.id, CYAN),
        ("키", doc.key, NEON),
        ("분류", "517.9 미분방정식 (UDC)", CYAN),
        ("파일", "~/.libran/library/Lee2023.pdf", FG_DIM),
        ("출처", "PDF 자체 추출", FG_DIM),
    ];

    for (label, value, color) in fields {
        lines.push(Line::from(vec![
            Span::styled(
                format!("[{:>6}]", label),
                Style::default().fg(NEON).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(value, Style::default().fg(color)),
        ]));
        lines.push(Line::from(""));
    }

    // Rating
    let stars = if doc.rating > 0 {
        "★".repeat(doc.rating as usize) + &"☆".repeat(5 - doc.rating as usize)
    } else {
        "☆".repeat(5)
    };
    lines.push(Line::from(vec![
        Span::styled(
            "[  별점]",
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(stars, Style::default().fg(YELLOW)),
    ]));
    lines.push(Line::from(""));

    // Abstract
    lines.push(Line::from(vec![Span::styled(
        "┏━━ ABSTRACT ━━┓",
        Style::default().fg(if focused { NEON } else { BORDER }),
    )]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "비선형 편미분방정식의 해법을 다룬 논문으로,",
            Style::default().fg(FG_DIM),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "분해법과 수치 해석을 결합한 접근을 제시한다.",
            Style::default().fg(FG_DIM),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "특히 반응-확산 계의 해 존재성을 증명하며...",
            Style::default().fg(FG_DIM),
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
fn render_status_bar(f: &mut Frame, area: Rect, focus: &Focus, _show_detail: bool) {
    let focus_label = match focus {
        Focus::Tree => "TREE",
        Focus::Docs => "DOCS",
        Focus::Detail => "DETAIL",
    };

    let hints = [
        ("Tab", "PANEL"),
        ("j/k", "NAV"),
        ("/", "SEARCH"),
        ("Enter", "DETAIL"),
        ("?", "HELP"),
        ("q", "QUIT"),
    ];
    let right_len: usize = hints
        .iter()
        .map(|(k, l)| k.len() + l.len() + 3)
        .sum::<usize>()
        + (hints.len() - 1) * 2;

    let left = vec![
        Span::styled("┃", Style::default().fg(NEON)),
        Span::raw(" "),
        Span::styled("●", Style::default().fg(NEON)),
        Span::styled(" READY", Style::default().fg(FG)),
        Span::raw("  "),
        Span::styled("┃", Style::default().fg(BORDER)),
        Span::raw("  "),
        Span::styled(
            focus_label,
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("┃", Style::default().fg(BORDER)),
        Span::raw("  "),
        Span::styled("v0.1", Style::default().fg(FG_DIM)),
    ];
    let left_len: usize = left.iter().map(|s| s.width()).sum();
    let pad = area.width as usize;
    let gap = pad.saturating_sub(left_len + right_len + 2);

    let mut spans = left;
    spans.push(Span::raw(" ".repeat(gap)));
    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(*key, Style::default().fg(NEON)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::default().fg(FG_DIM)));
    }
    spans.push(Span::raw(" "));
    spans.push(Span::styled("┃", Style::default().fg(BORDER)));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        area,
    );
}

// ── Search overlay ────────────────────────────────────────────
fn render_search_overlay(f: &mut Frame, area: Rect, input: &str) {
    let overlay = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: 3,
    };
    f.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(NEON))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(overlay);
    f.render_widget(block, overlay);

    let line = Line::from(vec![
        Span::styled("▶ ", Style::default().fg(NEON).add_modifier(Modifier::BOLD)),
        Span::styled(input, Style::default().fg(FG)),
        Span::styled("▎", Style::default().fg(NEON)),
    ]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(BG_PANEL)),
        inner,
    );
}

// ── Help overlay ──────────────────────────────────────────────
fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup = centered_rect(54, 72, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(NEON))
        .style(Style::default().bg(BG_PANEL))
        .title(Span::styled(
            " HELP ",
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            Style::default().fg(BORDER),
        )]),
        Line::from(vec![
            Span::styled("┃ ", Style::default().fg(BORDER)),
            Span::styled(
                "KEYBINDINGS",
                Style::default().fg(NEON).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ┃", Style::default().fg(BORDER)),
        ]),
        Line::from(vec![Span::styled(
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            Style::default().fg(BORDER),
        )]),
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
        help_line("q / Esc", "종료 (상세 보기 중일 시 닫기)"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[TIP] ",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "PDF 파일을 터미널로 드래그하여 추가",
                Style::default().fg(FG_DIM),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc / ?", Style::default().fg(FG_DIM)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(FG_DIM)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(BG_PANEL).fg(FG))
            .wrap(Wrap { trim: false }),
        inner,
    );
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("[{:>8}]", key),
            Style::default().fg(NEON).add_modifier(Modifier::BOLD),
        ),
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
