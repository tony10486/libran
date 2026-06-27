//! Design E — "Linear" Minimal Professional
//!
//! Inspired by Linear, Notion, and the Catppuccin Mocha palette.
//! Borderless design with soft pastel accents on a warm-dark surface.
//! Sections separated by subtle background color shifts, not lines.
//! Focus indicated by a left accent bar (▎) and soft row highlight.

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

// ── Catppuccin Mocha palette ──────────────────────────────────
const BASE: Color = Color::Rgb(30, 30, 46); // #1e1e2e
const SURFACE: Color = Color::Rgb(24, 24, 37); // #181825 (left panel bg)
const TEXT: Color = Color::Rgb(205, 214, 244); // #cdd6f4
const SUBTEXT: Color = Color::Rgb(166, 173, 200); // #a6adc8
const OVERLAY: Color = Color::Rgb(108, 112, 134); // #6c7086 (dim)
const MAUVE: Color = Color::Rgb(203, 166, 247); // #cba6f7 (accent)
const BLUE: Color = Color::Rgb(137, 180, 250); // #89b4fa
const YELLOW: Color = Color::Rgb(249, 226, 175); // #f9e2af
const GREEN: Color = Color::Rgb(166, 227, 161); // #a6e3a1
const RED: Color = Color::Rgb(243, 139, 168); // #f38ba8
const PINK: Color = Color::Rgb(245, 194, 231); // #f5c2e7
const TEAL: Color = Color::Rgb(148, 226, 213); // #94e2d5
const PEACH: Color = Color::Rgb(250, 179, 135); // #fab387

const FOCUS_BG: Color = Color::Rgb(49, 50, 68); // #313244 (surface0)
const DIVIDER: Color = Color::Rgb(49, 50, 68); // #313244

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

// ── Mock data ─────────────────────────────────────────────────
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
}

#[derive(Clone, Copy)]
enum ReadStatus {
    Unread,
    Reading,
    Read,
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
    },
];

const PROJECTS: &[(&str, u8, ReadStatus)] = &[
    ("ML 가속 연구", 2, ReadStatus::Reading),
    ("CUI 렌더러 설계", 1, ReadStatus::Read),
    ("수학적 모델링", 3, ReadStatus::Reading),
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
                    if focus == Focus::Docs {
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

    // Fill base background
    f.render_widget(Paragraph::new("").style(Style::default().bg(BASE)), area);

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

    render_status_bar(f, chunks[2], focus, show_detail, selected_docs.len());

    if *mode == Mode::Search {
        render_search_overlay(f, area, search_input);
    }
    if *mode == Mode::Help {
        render_help_overlay(f, area);
    }
}

// ── Header ────────────────────────────────────────────────────
fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "Libran",
            Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(OVERLAY)),
        Span::raw("  "),
        Span::styled("수학적 모델링", Style::default().fg(SUBTEXT)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(OVERLAY)),
        Span::raw("  "),
        Span::styled(
            "6",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" 문헌  "),
        Span::styled("·", Style::default().fg(OVERLAY)),
        Span::raw("  "),
        Span::styled("●", Style::default().fg(GREEN)),
        Span::styled(" 오프라인", Style::default().fg(SUBTEXT)),
    ]))
    .style(Style::default().bg(BASE));
    f.render_widget(header, area);
}

// ── List view (tree + docs) ────────────────────────────────────
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
        .constraints([Constraint::Percentage(30), Constraint::Min(1)])
        .split(area);

    render_tree(f, body[0], tree_state, focus == Focus::Tree, expanded_udc);
    render_doc_list(f, body[1], doc_state, focus == Focus::Docs, selected_docs);
}

fn render_tree(
    f: &mut Frame,
    area: Rect,
    state: &mut ListState,
    focused: bool,
    expanded_udc: &[&str],
) {
    let bg = SURFACE;
    let mut items: Vec<ListItem> = Vec::new();

    // Projects section
    items.push(section_header("프로젝트", bg));
    for (name, count, status) in PROJECTS {
        let dot = match status {
            ReadStatus::Read => GREEN,
            ReadStatus::Reading => YELLOW,
            ReadStatus::Unread => OVERLAY,
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("●", Style::default().fg(dot)),
            Span::raw(" "),
            Span::styled(*name, Style::default().fg(TEXT)),
            Span::styled(format!("  {}", count), Style::default().fg(OVERLAY)),
        ])));
    }

    items.push(ListItem::new(""));

    // UDC section
    items.push(section_header("UDC 분류", bg));
    for (notation, label, count, children) in UDC_TREE {
        let is_expanded = expanded_udc.contains(notation);
        let arrow = if is_expanded { "▾" } else { "▸" };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(arrow, Style::default().fg(OVERLAY)),
            Span::raw(" "),
            Span::styled(format!("{:<5}", notation), Style::default().fg(BLUE)),
            Span::styled(*label, Style::default().fg(TEXT)),
            if *count > 0 {
                Span::styled(format!("  {}", count), Style::default().fg(OVERLAY))
            } else {
                Span::raw("")
            },
        ])));

        if is_expanded {
            for (c_notation, c_label, c_count) in children.iter() {
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(format!("{:<7}", c_notation), Style::default().fg(BLUE)),
                    Span::styled(*c_label, Style::default().fg(SUBTEXT)),
                    if *c_count > 0 {
                        Span::styled(format!("  {}", c_count), Style::default().fg(OVERLAY))
                    } else {
                        Span::raw("")
                    },
                ])));
            }
        }
    }

    let highlight = if focused {
        Style::default().bg(FOCUS_BG).fg(TEXT)
    } else {
        Style::default().bg(bg)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(bg).fg(TEXT))
        .highlight_style(highlight)
        .highlight_symbol("▎ ");

    f.render_stateful_widget(list, area, state);
}

fn section_header(title: &str, bg: Color) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::styled("  ", Style::default().bg(bg)),
        Span::styled(
            title.to_uppercase(),
            Style::default().fg(OVERLAY).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("─".repeat(15), Style::default().fg(DIVIDER)),
    ]))
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
        let (dot, dot_color) = match doc.status {
            ReadStatus::Read => ("●", GREEN),
            ReadStatus::Reading => ("◑", YELLOW),
            ReadStatus::Unread => ("○", OVERLAY),
        };

        let check = if is_selected { "✓" } else { " " };
        let check_color = if is_selected { MAUVE } else { OVERLAY };

        let rating_str = if doc.rating > 0 {
            format!(" {}", "★".repeat(doc.rating as usize))
        } else {
            String::new()
        };

        // Line 1: status dot + title + rating
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(check, Style::default().fg(check_color)),
            Span::raw(" "),
            Span::styled(dot, Style::default().fg(dot_color)),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            if !rating_str.is_empty() {
                Span::styled(rating_str, Style::default().fg(YELLOW))
            } else {
                Span::raw("")
            },
        ])));

        // Line 2: metadata — authors, year, code, key
        let id_color = if doc.id.starts_with("doi:") {
            SUBTEXT
        } else {
            TEAL
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("      "),
            Span::styled(doc.authors, Style::default().fg(SUBTEXT)),
            Span::raw("  "),
            Span::styled(doc.year, Style::default().fg(SUBTEXT)),
            Span::raw("  "),
            Span::styled(doc.code, Style::default().fg(BLUE)),
            Span::raw("  "),
            Span::styled(doc.id, Style::default().fg(id_color)),
        ])));

        // Line 3: tags
        if !doc.tags.is_empty() {
            let mut tag_spans = vec![Span::raw("      ")];
            for (j, tag) in doc.tags.iter().enumerate() {
                if j > 0 {
                    tag_spans.push(Span::raw(" "));
                }
                tag_spans.push(Span::styled(
                    format!(" {} ", tag),
                    Style::default().fg(PINK),
                ));
            }
            items.push(ListItem::new(Line::from(tag_spans)));
        }

        items.push(ListItem::new(""));
    }

    let highlight = if focused {
        Style::default().bg(FOCUS_BG).fg(TEXT)
    } else {
        Style::default().bg(BASE)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(BASE).fg(TEXT))
        .highlight_style(highlight)
        .highlight_symbol("▎ ");

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
        .constraints([Constraint::Percentage(40), Constraint::Min(1)])
        .split(area);

    // Left: compact doc list (left panel bg for visual distinction)
    render_detail_doc_list(f, body[0], doc_state, focus == Focus::Docs);
    render_detail_panel(f, body[1], focus == Focus::Detail);
}

fn render_detail_doc_list(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let mut items: Vec<ListItem> = Vec::new();
    for doc in DOCS {
        let (dot, dot_color) = match doc.status {
            ReadStatus::Read => ("●", GREEN),
            ReadStatus::Reading => ("◑", YELLOW),
            ReadStatus::Unread => ("○", OVERLAY),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(dot, Style::default().fg(dot_color)),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(doc.authors, Style::default().fg(OVERLAY)),
            Span::raw(" "),
            Span::styled(doc.year, Style::default().fg(OVERLAY)),
        ])));
        items.push(ListItem::new(""));
    }

    let highlight = if focused {
        Style::default().bg(FOCUS_BG).fg(TEXT)
    } else {
        Style::default().bg(SURFACE)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .highlight_style(highlight)
        .highlight_symbol("▎ ");

    f.render_stateful_widget(list, area, state);
}

fn render_detail_panel(f: &mut Frame, area: Rect, focused: bool) {
    let doc = &DOCS[2]; // Lee2023 as example

    let (dot, dot_color) = match doc.status {
        ReadStatus::Read => ("●", GREEN),
        ReadStatus::Reading => ("◑", YELLOW),
        ReadStatus::Unread => ("○", OVERLAY),
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default().bg(BASE)),
            Span::styled(dot, Style::default().fg(dot_color)),
            Span::raw(" "),
            Span::styled(
                doc.title,
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(doc.authors, Style::default().fg(SUBTEXT)),
            Span::raw("  ·  "),
            Span::styled(doc.year, Style::default().fg(SUBTEXT)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ──────────────────────────────",
            Style::default().fg(DIVIDER),
        )]),
        Line::from(""),
    ];

    // Field rows
    let fields = [
        ("저자", doc.authors, TEXT),
        ("저널", "SIAM J. Math. Anal.", SUBTEXT),
        ("연도", doc.year, SUBTEXT),
        ("DOI", doc.id, TEAL),
        ("키", doc.key, GREEN),
        ("분류", "517.9 미분방정식 (UDC)", BLUE),
        ("파일", "~/.libran/library/Lee2023.pdf", OVERLAY),
        ("출처", "PDF 자체 추출", OVERLAY),
    ];

    for (label, value, color) in fields {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<6}", label), Style::default().fg(OVERLAY)),
            Span::raw("  "),
            Span::styled(value, Style::default().fg(color)),
        ]));
        lines.push(Line::from(""));
    }

    // Tags
    let mut tag_line = vec![
        Span::raw("  "),
        Span::styled("태그  ", Style::default().fg(OVERLAY)),
        Span::raw("  "),
    ];
    for (i, tag) in doc.tags.iter().enumerate() {
        if i > 0 {
            tag_line.push(Span::raw(" "));
        }
        tag_line.push(Span::styled(
            format!(" {} ", tag),
            Style::default().fg(PINK),
        ));
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
        Span::raw("  "),
        Span::styled("별점  ", Style::default().fg(OVERLAY)),
        Span::raw("  "),
        Span::styled(stars, Style::default().fg(YELLOW)),
    ]));
    lines.push(Line::from(""));

    // Abstract
    lines.push(Line::from(vec![Span::styled(
        "  ─── 초록 ───",
        Style::default().fg(OVERLAY),
    )]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "비선형 편미분방정식의 해법을 다룬 논문으로,",
            Style::default().fg(SUBTEXT),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "분해법과 수치 해석을 결합한 접근을 제시한다.",
            Style::default().fg(SUBTEXT),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "특히 반응-확산 계의 해 존재성을 증명하며...",
            Style::default().fg(OVERLAY),
        ),
    ]));

    let style = if focused {
        Style::default().bg(BASE).fg(TEXT)
    } else {
        Style::default().bg(BASE).fg(SUBTEXT)
    };

    let para = Paragraph::new(lines)
        .style(style)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

// ── Status bar ────────────────────────────────────────────────
fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    focus: &Focus,
    _show_detail: bool,
    sel_count: usize,
) {
    let focus_label = match focus {
        Focus::Tree => "트리",
        Focus::Docs => "문헌",
        Focus::Detail => "상세",
    };

    let mut spans = vec![
        Span::raw(" "),
        Span::styled("준비됨", Style::default().fg(GREEN)),
        Span::raw("  "),
        Span::styled("·", Style::default().fg(OVERLAY)),
        Span::raw("  "),
        Span::styled(focus_label, Style::default().fg(MAUVE)),
    ];

    if sel_count > 0 {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("·", Style::default().fg(OVERLAY)));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{} 선택", sel_count),
            Style::default().fg(YELLOW),
        ));
    }

    // Right-aligned hints
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

    let left_len: usize = spans.iter().map(|s| s.width()).sum();
    let gap = area.width as usize;
    let pad = gap.saturating_sub(left_len + right_len + 2);
    spans.push(Span::raw(" ".repeat(pad)));

    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(*key, Style::default().fg(MAUVE)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::default().fg(OVERLAY)));
    }

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(BASE));
    f.render_widget(bar, area);
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

    // Background bar
    let bg_bar = Rect {
        height: 3,
        ..overlay
    };
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(SURFACE)),
        bg_bar,
    );

    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("⌕", Style::default().fg(MAUVE).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(TEXT)),
        Span::styled("▎", Style::default().fg(MAUVE)),
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
                Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("─".repeat(20), Style::default().fg(DIVIDER)),
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
        help_line("q / Esc", "종료 (상세 보기 중일 시 닫기)"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Drag & Drop",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "PDF 파일을 터미널로 드래그하여 추가",
                Style::default().fg(SUBTEXT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc / ?", Style::default().fg(OVERLAY)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(OVERLAY)),
        ]),
    ];

    let para = Paragraph::new(lines)
        .style(Style::default().bg(SURFACE).fg(TEXT))
        .wrap(Wrap { trim: false });
    f.render_widget(para, popup);
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<8}", key), Style::default().fg(MAUVE)),
        Span::raw(" "),
        Span::styled(desc.to_string(), Style::default().fg(SUBTEXT)),
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
