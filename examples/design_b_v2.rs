use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, stdout};

#[derive(PartialEq)]
enum Focus { Tree, Docs, Detail }

#[derive(PartialEq)]
enum Mode { Normal, Search, Help }

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tree_state = ListState::default();
    tree_state.select(Some(5));
    let mut doc_state = ListState::default();
    doc_state.select(Some(0));

    let mut focus = Focus::Tree;
    let mut mode = Mode::Normal;
    let mut search_input = String::new();
    let mut show_detail = false;
    let mut selected_idx: Option<usize> = None;

    loop {
        terminal.draw(|f| {
            render(f, &mut tree_state, &mut doc_state, &focus, &mode, &search_input, show_detail, selected_idx);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            if mode == Mode::Help {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => mode = Mode::Normal,
                    _ => {}
                }
                continue;
            }

            if mode == Mode::Search {
                match key.code {
                    KeyCode::Esc => { mode = Mode::Normal; search_input.clear(); }
                    KeyCode::Enter => { mode = Mode::Normal; }
                    KeyCode::Backspace => { search_input.pop(); }
                    KeyCode::Char(c) => { search_input.push(c); }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if show_detail { show_detail = false; } else { break; }
                }
                KeyCode::Tab => {
                    focus = match focus {
                        Focus::Tree => Focus::Docs,
                        Focus::Docs if show_detail => Focus::Detail,
                        Focus::Docs => Focus::Tree,
                        Focus::Detail => Focus::Tree,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match focus {
                        Focus::Tree => tree_state.select_next(),
                        Focus::Docs if !show_detail => doc_state.select_next(),
                        _ => {}
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    match focus {
                        Focus::Tree => tree_state.select_previous(),
                        Focus::Docs if !show_detail => doc_state.select_previous(),
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    if focus == Focus::Docs && !show_detail {
                        selected_idx = doc_state.selected();
                        show_detail = true;
                        focus = Focus::Docs;
                    } else if show_detail {
                        show_detail = false;
                    }
                }
                KeyCode::Char('/') => { mode = Mode::Search; search_input.clear(); }
                KeyCode::Char('?') => { mode = Mode::Help; }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

const SURFACE: Color = Color::Black;
const PRIMARY: Color = Color::Cyan;
const PRIMARY_DIM: Color = Color::DarkGray;
const ACCENT: Color = Color::Yellow;
const ACCENT2: Color = Color::Green;
const TEXT: Color = Color::White;
const TEXT_DIM: Color = Color::Gray;
const TEXT_FAINT: Color = Color::DarkGray;

fn render(
    f: &mut Frame,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
    show_detail: bool,
    selected_idx: Option<usize>,
) {
    let area = f.area();

    if show_detail {
        render_detail_mode(f, area, doc_state, focus, mode, search_input, selected_idx);
    } else {
        render_list_mode(f, area, tree_state, doc_state, focus, mode, search_input);
    }

    if *mode == Mode::Search {
        render_search_overlay(f, area, search_input);
    }

    if *mode == Mode::Help {
        render_help_overlay(f, area);
    }
}

fn render_list_mode(
    f: &mut Frame,
    area: Rect,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(f, chunks[0], search_input, *mode == Mode::Search);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    render_tree(f, body[0], tree_state, *focus == Focus::Tree);
    render_doc_list(f, body[1], doc_state, *focus == Focus::Docs);

    render_footer(f, chunks[2]);
}

fn render_detail_mode(
    f: &mut Frame,
    area: Rect,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
    selected_idx: Option<usize>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(f, chunks[0], search_input, *mode == Mode::Search);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    render_doc_list(f, body[0], doc_state, *focus == Focus::Docs);
    render_detail_panel(f, body[1], selected_idx, *focus == Focus::Detail);

    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect, _search_input: &str, _is_searching: bool) {
    let header = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("Libran", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(PRIMARY_DIM)),
        Span::raw("  "),
        Span::styled("머신러닝 가속 연구", Style::default().fg(TEXT)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(PRIMARY_DIM)),
        Span::raw("  "),
        Span::styled("4", Style::default().fg(ACCENT)),
        Span::raw(" 문헌  "),
        Span::styled("│", Style::default().fg(PRIMARY_DIM)),
        Span::raw("  "),
        Span::styled("● 오프라인", Style::default().fg(ACCENT2)),
    ]))
    .style(Style::default().bg(SURFACE));
    f.render_widget(header, area);
}

fn render_tree(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let border_color = if focused { PRIMARY } else { PRIMARY_DIM };

    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  프로젝트", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("ML 가속", Style::default().fg(TEXT)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("CUI 렌더러", Style::default().fg(TEXT)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  UDC 분류", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("0  ", Style::default().fg(ACCENT)),
            Span::styled("총류·정보학", Style::default().fg(TEXT)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("5  ", Style::default().fg(ACCENT)),
            Span::styled("자연과학", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(12)", Style::default().fg(TEXT_DIM)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    ▾ "),
            Span::styled("51 ", Style::default().fg(ACCENT)),
            Span::styled("수학", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(5)", Style::default().fg(TEXT_DIM)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("      ▸ "),
            Span::styled("512 ", Style::default().fg(ACCENT)),
            Span::styled("대수", Style::default().fg(TEXT)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("      ▾ "),
            Span::styled("517 ", Style::default().fg(ACCENT)),
            Span::styled("해석학", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(3)", Style::default().fg(TEXT_DIM)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("        "),
            Span::styled("517.9 ", Style::default().fg(ACCENT)),
            Span::styled("미분방정식", Style::default().fg(TEXT)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    ▸ "),
            Span::styled("53 ", Style::default().fg(ACCENT)),
            Span::styled("물리학", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(4)", Style::default().fg(TEXT_DIM)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("6  ", Style::default().fg(ACCENT)),
            Span::styled("응용과학", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(7)", Style::default().fg(TEXT_DIM)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  PhySH", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("응축물질", Style::default().fg(TEXT)),
            Span::raw(" "),
            Span::styled("(3)", Style::default().fg(TEXT_DIM)),
        ])),
    ];

    let list = List::default()
        .items(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(SURFACE)),
        )
        .highlight_style(
            Style::default()
                .bg(PRIMARY)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, state);
}

fn render_doc_list(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let border_color = if focused { PRIMARY } else { PRIMARY_DIM };

    let items: Vec<ListItem> = vec![
        doc_item("517.9", "Nonisothermal Diffuse Interface Model", "Smith, J. (2024)", "doi:10.33048/SIBJIM.2022.25.103", "Smith2024", false),
        ListItem::new(""),
        doc_item("51-72", "Mathematical Modeling in Physics", "Kim, D. (2023)", "doi:10.1006/jmbi.2023.2354", "Kim2023", false),
        ListItem::new(""),
        doc_item("35-XX", "PDE Solutions for Nonlinear Systems", "Lee, S. · Park, M. (2023)", "arXiv:2301.00123", "Lee2023", true),
        ListItem::new(""),
        doc_item("53", "Quantum Hall Effect in Graphene", "Zhang, Y. (2024)", "doi:10.1103/PhysRevLett.132.046", "Zhang2024", false),
    ];

    let list = List::default()
        .items(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(SURFACE))
                .title_top(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("문헌", Style::default().fg(TEXT)),
                    Span::raw(" "),
                    Span::styled("4", Style::default().fg(ACCENT)),
                    Span::raw(" "),
                ])),
        )
        .highlight_style(
            Style::default()
                .bg(PRIMARY)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, state);
}

fn doc_item<'a>(code: &'a str, title: &'a str, author: &'a str, id: &'a str, key: &'a str, selected: bool) -> ListItem<'a> {
    let marker = if selected { "◆ " } else { "  " };
    let marker_style = if selected { Style::default().fg(ACCENT) } else { Style::default() };

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(marker, marker_style),
            Span::styled(format!("{:<6} ", code), Style::default().fg(ACCENT)),
            Span::styled(title, Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("          "),
            Span::styled(author, Style::default().fg(TEXT_DIM)),
            Span::raw("  "),
            Span::styled(id, Style::default().fg(TEXT_FAINT)),
            Span::raw("  "),
            Span::styled(format!("[{}]", key), Style::default().fg(ACCENT2)),
        ]),
    ])
}

fn render_detail_panel(f: &mut Frame, area: Rect, _selected_idx: Option<usize>, focused: bool) {
    let border_color = if focused { PRIMARY } else { PRIMARY_DIM };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  제목   ", Style::default().fg(PRIMARY)),
            Span::styled("PDE Solutions for Nonlinear Systems", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  저자   ", Style::default().fg(PRIMARY)), Span::styled("Lee, S. · Park, M.", Style::default().fg(TEXT))]),
        Line::from(""),
        Line::from(vec![Span::styled("  저널   ", Style::default().fg(PRIMARY)), Span::styled("—", Style::default().fg(TEXT_DIM))]),
        Line::from(""),
        Line::from(vec![Span::styled("  연도   ", Style::default().fg(PRIMARY)), Span::styled("2023", Style::default().fg(TEXT))]),
        Line::from(""),
        Line::from(vec![Span::styled("  DOI    ", Style::default().fg(PRIMARY)), Span::styled("—", Style::default().fg(TEXT_DIM))]),
        Line::from(""),
        Line::from(vec![Span::styled("  arXiv  ", Style::default().fg(PRIMARY)), Span::styled("2301.00123", Style::default().fg(Color::Blue))]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  분류   ", Style::default().fg(PRIMARY)),
            Span::styled("517.9 ", Style::default().fg(ACCENT)),
            Span::styled("미분방정식  ", Style::default().fg(TEXT)),
            Span::styled("(UDC)", Style::default().fg(TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::raw("         "),
            Span::styled("35-XX ", Style::default().fg(ACCENT2)),
            Span::styled("편미분방정식  ", Style::default().fg(TEXT)),
            Span::styled("(MSC)", Style::default().fg(TEXT_DIM)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  키     ", Style::default().fg(PRIMARY)), Span::styled("Lee2023", Style::default().fg(ACCENT2))]),
        Line::from(""),
        Line::from(vec![Span::styled("  파일   ", Style::default().fg(PRIMARY)), Span::styled("~/.libran/library/Lee2023.pdf", Style::default().fg(TEXT_DIM))]),
        Line::from(""),
        Line::from(vec![Span::styled("  출처   ", Style::default().fg(PRIMARY)), Span::styled("PDF 자체 추출", Style::default().fg(TEXT_DIM))]),
        Line::from(""),
        Line::from(vec![Span::styled("  ──────── 초록 ────────", Style::default().fg(PRIMARY_DIM))]),
        Line::from(""),
        Line::from(vec![Span::raw("  " ), Span::styled("비선형 편미분방정식의 해법을 다룬 논문으로,", Style::default().fg(TEXT))]),
        Line::from(vec![Span::raw("  " ), Span::styled("분해법과 수치 해석을 결합한 접근을 제시한다.", Style::default().fg(TEXT))]),
        Line::from(vec![Span::raw("  " ), Span::styled("특히 반응-확산 계의 해 존재성을 증명하며...", Style::default().fg(TEXT_DIM))]),
    ];

    let detail = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(SURFACE))
                .title_top(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("상세", Style::default().fg(TEXT)),
                    Span::raw(" "),
                ])),
        );

    f.render_widget(detail, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let keys = vec![
        (" Tab", "패널"),
        (" j/k", "탐색"),
        (" Enter", "상세"),
        (" /", "검색"),
        (" Space", "선택"),
        (" x", "내보내기"),
        (" ?", "도움말"),
        (" q", "종료"),
    ];

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, label)) in keys.iter().enumerate() {
        if i > 0 { spans.push(Span::raw("  ")); }
        spans.push(Span::styled(*key, Style::default().fg(PRIMARY)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::default().fg(TEXT_DIM)));
    }

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(SURFACE));
    f.render_widget(footer, area);
}

fn render_search_overlay(f: &mut Frame, area: Rect, input: &str) {
    let overlay_area = Rect {
        x: area.x + 3,
        y: area.y,
        width: area.width.saturating_sub(6),
        height: 3,
    };

    f.render_widget(Clear, overlay_area);

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PRIMARY))
        .style(Style::default().bg(Color::Black));

    let search_line = Line::from(vec![
        Span::raw(" "),
        Span::styled("🔍", Style::default().fg(PRIMARY)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(TEXT)),
        Span::styled("▎", Style::default().fg(PRIMARY)),
    ]);

    let search = Paragraph::new(search_line).block(search_block);
    f.render_widget(search, overlay_area);
}

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 70, area);
    f.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  단축키", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab      ", Style::default().fg(PRIMARY)),
            Span::styled("패널 간 포커스 이동", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j / ↓    ", Style::default().fg(PRIMARY)),
            Span::styled("아래로 이동", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  k / ↑    ", Style::default().fg(PRIMARY)),
            Span::styled("위로 이동", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter    ", Style::default().fg(PRIMARY)),
            Span::styled("문헌 상세 보기 / 닫기", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  /        ", Style::default().fg(PRIMARY)),
            Span::styled("검색 모드 진입", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Space    ", Style::default().fg(PRIMARY)),
            Span::styled("문헌 다중 선택 토글", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  x        ", Style::default().fg(PRIMARY)),
            Span::styled("BibTeX / CSL JSON 내보내기", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  n        ", Style::default().fg(PRIMARY)),
            Span::styled("새 프로젝트 생성", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?        ", Style::default().fg(PRIMARY)),
            Span::styled("도움말 토글", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q / Esc  ", Style::default().fg(PRIMARY)),
            Span::styled("종료 (상세 보기 중일 시 상세 닫기)", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Drag & Drop", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("PDF 파일을 터미널로 드래그하여 추가", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ESC / ?", Style::default().fg(TEXT_DIM)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(TEXT_DIM)),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY))
                .style(Style::default().bg(Color::Black))
                .title_top(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("Libran — 도움말", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                ])),
        );
    f.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}
