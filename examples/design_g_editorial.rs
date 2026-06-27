//! Design G — "Editorial" Warm Research Library
//!
//! Inspired by classic research library aesthetics — warm sepia tones,
//! ornamental dividers, and a reading-focused column layout. Feels like
//! a leather-bound journal or a vintage card catalog. Amber accents on
//! dark warm-brown backgrounds, with serif-like decorative headers.

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

// ── Warm Editorial palette ────────────────────────────────────
const BG: Color = Color::Rgb(38, 34, 28); // #26221c — dark warm brown
const SURFACE: Color = Color::Rgb(32, 28, 23); // #201c17 — left panel
const FG: Color = Color::Rgb(224, 200, 148); // #e0c894 — warm cream
const FG_DIM: Color = Color::Rgb(160, 140, 100); // #a08c64 — muted amber
const FAINT: Color = Color::Rgb(100, 88, 65); // #645841 — very dim
const AMBER: Color = Color::Rgb(212, 166, 87); // #d4a657 — amber accent
const GREEN: Color = Color::Rgb(168, 184, 106); // #a8b86a — sage green
const ROSE: Color = Color::Rgb(201, 165, 212); // #c9a5d4 — muted lavender
const BLUE: Color = Color::Rgb(130, 165, 200); // #82a5c8 — dusty blue
const RED: Color = Color::Rgb(200, 120, 100); // #c87864 — terracotta
const FOCUS_BG: Color = Color::Rgb(48, 42, 35); // #302a23 — focus highlight
const ORNAMENT: Color = Color::Rgb(90, 78, 58); // #5a4e3a — decorative lines

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
        tags: &["PDE", "nonlinear"],
        journal: "arXiv preprint",
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
        tags: &["ML", "edge"],
        journal: "arXiv preprint",
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
        tags: &["algebra", "quantum"],
        journal: "J. Appl. Math.",
    },
];

const PROJECTS: &[(&str, u8, ReadStatus)] = &[
    ("ML 가속 연구", 2, ReadStatus::Reading),
    ("CUI 렌더러 설계", 1, ReadStatus::Read),
    ("수학적 모델링", 3, ReadStatus::Reading),
];

const UDC_TREE: &[(&str, &str, u8, &[(&str, &str, u8)])] = &[
    ("0", "총류 · 정보학", 1, &[("004", "컴퓨터과학", 1)]),
    ("1", "철학 · 심리학", 0, &[]),
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
    ("9", "역사 · 지리", 0, &[]),
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

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

// ── Header — ornamental title ─────────────────────────────────
fn render_header(f: &mut Frame, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    // Title row
    let title = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("✦", Style::default().fg(AMBER)),
        Span::raw(" "),
        Span::styled(
            "Libran",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("─ Bibliotheca ─", Style::default().fg(ORNAMENT)),
        Span::raw("  "),
        Span::styled("수학적 모델링", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("✦", Style::default().fg(AMBER)),
    ]))
    .style(Style::default().bg(BG));
    f.render_widget(title, rows[0]);

    // Ornamental divider
    let div = Paragraph::new(Line::from(vec![
        Span::styled(
            " ════════════════════════════════════════════════════════════════",
            Style::default().fg(ORNAMENT),
        ),
        Span::raw("  "),
        Span::styled("6 문헌", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("3 읽음", Style::default().fg(GREEN)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("2 읽는중", Style::default().fg(AMBER)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("1 안읽음", Style::default().fg(FG_DIM)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled("● 오프라인", Style::default().fg(GREEN)),
    ]))
    .style(Style::default().bg(BG));
    f.render_widget(div, rows[1]);
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
        .constraints([Constraint::Percentage(28), Constraint::Min(1)])
        .split(area);

    render_tree(f, body[0], tree_state, focus == Focus::Tree, expanded_udc);
    render_doc_list(f, body[1], doc_state, focus == Focus::Docs);
}

fn render_tree(
    f: &mut Frame,
    area: Rect,
    state: &mut ListState,
    focused: bool,
    expanded_udc: &[&str],
) {
    let mut items: Vec<ListItem> = Vec::new();

    // Projects
    items.push(ornamental_header("프로젝트"));
    items.push(ListItem::new(""));

    for (name, count, status) in PROJECTS {
        let (glyph, color) = match status {
            ReadStatus::Read => ("❖", GREEN),
            ReadStatus::Reading => ("❖", AMBER),
            ReadStatus::Unread => ("❖", FAINT),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(*name, Style::default().fg(FG)),
            Span::raw(" "),
            Span::styled(format!("({})", count), Style::default().fg(FG_DIM)),
        ])));
        items.push(ListItem::new(""));
    }

    // UDC
    items.push(ornamental_header("UDC 분류"));
    items.push(ListItem::new(""));

    for (notation, label, count, children) in UDC_TREE {
        let is_expanded = expanded_udc.contains(notation);
        let arrow = if is_expanded { "▾" } else { "▸" };

        items.push(ListItem::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(arrow, Style::default().fg(ORNAMENT)),
            Span::raw(" "),
            Span::styled(format!("{:<5}", notation), Style::default().fg(BLUE)),
            Span::styled(*label, Style::default().fg(FG)),
            if *count > 0 {
                Span::styled(format!("  {}", count), Style::default().fg(AMBER))
            } else {
                Span::raw("")
            },
        ])));

        if is_expanded {
            for (c_notation, c_label, c_count) in children.iter() {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::styled("·", Style::default().fg(ORNAMENT)),
                    Span::raw(" "),
                    Span::styled(format!("{:<5}", c_notation), Style::default().fg(BLUE)),
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
        .highlight_symbol("❯ ");

    f.render_stateful_widget(list, area, state);
}

fn ornamental_header(title: &str) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("╔ ", Style::default().fg(ORNAMENT)),
        Span::styled(
            title.to_string(),
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ╗", Style::default().fg(ORNAMENT)),
    ]))
}

// ── Document list ─────────────────────────────────────────────
fn render_doc_list(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let mut items: Vec<ListItem> = Vec::new();

    // Ornamental column header
    items.push(ListItem::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("◆", Style::default().fg(AMBER)),
        Span::styled(
            "  문헌 목록  ",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::styled("◆", Style::default().fg(AMBER)),
    ])));
    items.push(ListItem::new(Line::from(vec![Span::styled(
        " ════════════════════════════════════════════════════════════════",
        Style::default().fg(ORNAMENT),
    )])));
    items.push(ListItem::new(""));

    for doc in DOCS {
        let (glyph, glyph_color) = match doc.status {
            ReadStatus::Read => ("❉", GREEN),
            ReadStatus::Reading => ("❀", AMBER),
            ReadStatus::Unread => ("○", FAINT),
        };

        let id_color = if doc.id.starts_with("doi:") {
            FG_DIM
        } else {
            BLUE
        };
        let rating_str = if doc.rating > 0 {
            format!("  {}", "★".repeat(doc.rating as usize))
        } else {
            String::new()
        };

        // Line 1: glyph + title + rating
        items.push(ListItem::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(glyph, Style::default().fg(glyph_color)),
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

        // Line 2: authors · year · journal · code
        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(AMBER)),
            Span::raw("  ·  "),
            Span::styled(doc.journal, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.code, Style::default().fg(BLUE)),
        ])));

        // Line 3: id + key + tags
        let mut line3 = vec![
            Span::raw("   "),
            Span::styled(doc.id, Style::default().fg(id_color)),
            Span::raw("  "),
            Span::styled(format!("[{}]", doc.key), Style::default().fg(GREEN)),
        ];
        if !doc.tags.is_empty() {
            line3.push(Span::raw("  "));
            for (i, tag) in doc.tags.iter().enumerate() {
                if i > 0 {
                    line3.push(Span::raw(" "));
                }
                line3.push(Span::styled(format!("#{}", tag), Style::default().fg(ROSE)));
            }
        }
        items.push(ListItem::new(Line::from(line3)));
        items.push(ListItem::new(""));
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
        .highlight_symbol("❯ ");

    f.render_stateful_widget(list, area, state);
}

// ── Detail view ───────────────────────────────────────────────
fn render_detail_view(f: &mut Frame, area: Rect, doc_state: &mut ListState, focus: Focus) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Min(1)])
        .split(area);

    // Compact list
    let mut items: Vec<ListItem> = Vec::new();
    for doc in DOCS {
        let (glyph, color) = match doc.status {
            ReadStatus::Read => ("❉", GREEN),
            ReadStatus::Reading => ("❀", AMBER),
            ReadStatus::Unread => ("○", FAINT),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::styled(doc.authors, Style::default().fg(FAINT)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(FAINT)),
        ])));
        items.push(ListItem::new(""));
    }

    let highlight = if focus == Focus::Docs {
        Style::default().bg(FOCUS_BG).fg(FG)
    } else {
        Style::default().bg(SURFACE)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(SURFACE).fg(FG))
        .highlight_style(highlight)
        .highlight_symbol("❯ ");
    f.render_stateful_widget(list, body[0], doc_state);

    render_detail_panel(f, body[1], focus == Focus::Detail);
}

fn render_detail_panel(f: &mut Frame, area: Rect, focused: bool) {
    let doc = &DOCS[2];

    let (glyph, glyph_color) = match doc.status {
        ReadStatus::Read => ("❉", GREEN),
        ReadStatus::Reading => ("❀", AMBER),
        ReadStatus::Unread => ("○", FAINT),
    };

    let mut lines = vec![
        Line::from(""),
        // Ornamental title
        Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "╔══════════════════════════════════════╗",
                Style::default().fg(ORNAMENT),
            ),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("║ ", Style::default().fg(ORNAMENT)),
            Span::styled(glyph, Style::default().fg(glyph_color)),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("║ ", Style::default().fg(ORNAMENT)),
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(FG_DIM)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(AMBER)),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "╚══════════════════════════════════════╝",
                Style::default().fg(ORNAMENT),
            ),
        ]),
        Line::from(""),
    ];

    // Field rows with ornamental labels
    let fields = [
        ("저자", doc.authors, FG),
        ("저널", doc.journal, FG_DIM),
        ("연도", doc.year, AMBER),
        ("DOI", doc.id, BLUE),
        ("키", doc.key, GREEN),
        ("분류", "517.9 미분방정식 (UDC)", BLUE),
        ("파일", "~/.libran/library/Lee2023.pdf", FG_DIM),
        ("출처", "PDF 자체 추출", FG_DIM),
    ];

    for (label, value, color) in fields {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("❖ ", Style::default().fg(AMBER)),
            Span::styled(
                format!("{:<6}", label),
                Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(value, Style::default().fg(color)),
        ]));
        lines.push(Line::from(""));
    }

    // Tags
    let mut tag_line = vec![
        Span::raw(" "),
        Span::styled("❖ ", Style::default().fg(AMBER)),
        Span::styled(
            "태그  ",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    for (i, tag) in doc.tags.iter().enumerate() {
        if i > 0 {
            tag_line.push(Span::raw("  "));
        }
        tag_line.push(Span::styled(format!("#{}", tag), Style::default().fg(ROSE)));
    }
    lines.push(Line::from(tag_line));
    lines.push(Line::from(""));

    // Rating
    let stars = if doc.rating > 0 {
        "★".repeat(doc.rating as usize) + &"☆".repeat(5 - doc.rating as usize)
    } else {
        "☆".repeat(5)
    };
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("❖ ", Style::default().fg(AMBER)),
        Span::styled(
            "별점  ",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(stars, Style::default().fg(AMBER)),
    ]));
    lines.push(Line::from(""));

    // Abstract
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("╔ ", Style::default().fg(ORNAMENT)),
        Span::styled(
            "초록",
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ╗", Style::default().fg(ORNAMENT)),
    ]));
    lines.push(Line::from(vec![Span::styled(
        " ════════════════════════════════════════",
        Style::default().fg(ORNAMENT),
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
fn render_status_bar(f: &mut Frame, area: Rect, focus: &Focus, _show_detail: bool) {
    let focus_label = match focus {
        Focus::Tree => "트리",
        Focus::Docs => "문헌",
        Focus::Detail => "상세",
    };

    let hints = [
        ("Tab", "패널"),
        ("j/k", "이동"),
        ("/", "검색"),
        ("Enter", "상세"),
        ("?", "도움말"),
        ("q", "종료"),
    ];
    let right_len: usize = hints
        .iter()
        .map(|(k, l)| k.len() + l.chars().count() + 3)
        .sum::<usize>()
        + (hints.len() - 1) * 2;

    let left = vec![
        Span::raw(" "),
        Span::styled("❖", Style::default().fg(GREEN)),
        Span::raw(" "),
        Span::styled("준비됨", Style::default().fg(FG)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(FAINT)),
        Span::raw("  "),
        Span::styled(focus_label, Style::default().fg(AMBER)),
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
        spans.push(Span::styled(*key, Style::default().fg(AMBER)));
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
        Span::styled("❖", Style::default().fg(AMBER).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(FG)),
        Span::styled("▎", Style::default().fg(AMBER)),
    ]);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(SURFACE)),
        overlay,
    );
}

// ── Help overlay ──────────────────────────────────────────────
fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 72, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(SURFACE)),
        popup,
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("╔ ", Style::default().fg(ORNAMENT)),
            Span::styled(
                "단축키",
                Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ╗", Style::default().fg(ORNAMENT)),
        ]),
        Line::from(vec![Span::styled(
            " ════════════════════════════════════════",
            Style::default().fg(ORNAMENT),
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
            Span::raw(" "),
            Span::styled("❖ ", Style::default().fg(AMBER)),
            Span::styled(
                "Drag & Drop",
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
            Span::raw(" "),
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
        Span::raw(" "),
        Span::styled(
            format!("{:<8}", key),
            Style::default().fg(AMBER).add_modifier(Modifier::BOLD),
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
