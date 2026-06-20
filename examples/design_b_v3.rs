use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
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

    loop {
        terminal.draw(|f| {
            render(f, &mut tree_state, &mut doc_state, &focus, &mode, &search_input, show_detail);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            match mode {
                Mode::Help => {
                    if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                        mode = Mode::Normal;
                    }
                    continue;
                }
                Mode::Search => {
                    match key.code {
                        KeyCode::Esc => { mode = Mode::Normal; search_input.clear(); }
                        KeyCode::Enter => { mode = Mode::Normal; }
                        KeyCode::Backspace => { search_input.pop(); }
                        KeyCode::Char(c) => { search_input.push(c); }
                        _ => {}
                    }
                    continue;
                }
                Mode::Normal => {}
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if show_detail { show_detail = false; } else { break; }
                }
                KeyCode::Tab => {
                    focus = match (&focus, show_detail) {
                        (Focus::Tree, _) => Focus::Docs,
                        (Focus::Docs, false) => Focus::Tree,
                        (Focus::Docs, true) => Focus::Detail,
                        (Focus::Detail, _) => Focus::Docs,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match &focus {
                        Focus::Tree => tree_state.select_next(),
                        Focus::Docs if !show_detail => doc_state.select_next(),
                        _ => {}
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    match &focus {
                        Focus::Tree => tree_state.select_previous(),
                        Focus::Docs if !show_detail => doc_state.select_previous(),
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    if focus == Focus::Docs {
                        show_detail = !show_detail;
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

fn render(
    f: &mut Frame,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
    mode: &Mode,
    search_input: &str,
    show_detail: bool,
) {
    let area = f.area();

    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    if show_detail {
        render_detail_view(f, area, doc_state, focus, search_input);
    } else {
        render_list_view(f, area, tree_state, doc_state, focus);
    }

    if *mode == Mode::Search {
        render_search_overlay(f, area, search_input);
    }

    if *mode == Mode::Help {
        render_help_overlay(f, area);
    }
}

fn render_list_view(
    f: &mut Frame,
    area: Rect,
    tree_state: &mut ListState,
    doc_state: &mut ListState,
    focus: &Focus,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(f, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Percentage(28), Constraint::Length(1), Constraint::Min(1)])
        .split(chunks[1]);

    render_tree(f, body[1], tree_state, *focus == Focus::Tree);
    render_vdivider(f, body[2]);
    render_doc_list(f, body[3], doc_state, *focus == Focus::Docs);

    render_footer(f, chunks[2]);
}

fn render_detail_view(
    f: &mut Frame,
    area: Rect,
    doc_state: &mut ListState,
    focus: &Focus,
    _search_input: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_header(f, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Length(1), Constraint::Min(1)])
        .split(chunks[1]);

    render_doc_list(f, body[0], doc_state, *focus == Focus::Docs);
    render_vdivider(f, body[1]);
    render_detail_panel(f, body[2], *focus == Focus::Detail);

    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("Libran", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("머신러닝 가속 연구", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("4", Style::default().fg(Color::Yellow)),
        Span::raw(" 문헌  "),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("● 오프라인", Style::default().fg(Color::Green)),
    ]))
    .style(Style::default().bg(Color::Black));
    f.render_widget(header, area);
}

fn render_tree(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  프로젝트", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("ML 가속", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled("CUI 렌더러", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  UDC 분류", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("0  ", Style::default().fg(Color::Yellow)),
            Span::styled("총류·정보학", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("5  ", Style::default().fg(Color::Yellow)),
            Span::styled("자연과학", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(12)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    ▾ "),
            Span::styled("51 ", Style::default().fg(Color::Yellow)),
            Span::styled("수학", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(5)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("      ▸ "),
            Span::styled("512 ", Style::default().fg(Color::Yellow)),
            Span::styled("대수", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("      ▾ "),
            Span::styled("517 ", Style::default().fg(Color::Yellow)),
            Span::styled("해석학", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(3)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("        "),
            Span::styled("517.9 ", Style::default().fg(Color::Yellow)),
            Span::styled("미분방정식", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("    ▸ "),
            Span::styled("53 ", Style::default().fg(Color::Yellow)),
            Span::styled("물리학", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(4)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("6  ", Style::default().fg(Color::Yellow)),
            Span::styled("응용과학", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(7)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  PhySH", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  ▸ "),
            Span::styled("응축물질", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled("(3)", Style::default().fg(Color::DarkGray)),
        ])),
    ];

    let highlight = if focused {
        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::Black).fg(Color::DarkGray)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(Color::Black))
        .highlight_style(highlight);

    f.render_stateful_widget(list, area, state);
}

fn render_vdivider(f: &mut Frame, area: Rect) {
    for y in area.top()..area.bottom() {
        let divider = Paragraph::new("│")
            .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
        f.render_widget(divider, Rect { x: area.x, y, width: 1, height: 1 });
    }
}

fn render_doc_list(f: &mut Frame, area: Rect, state: &mut ListState, focused: bool) {
    let items: Vec<ListItem> = vec![
        doc_item("517.9", "Nonisothermal Diffuse Interface Model", "Smith, J. (2024)", "doi:10.33048/SIBJIM.2022.25.103", "Smith2024", false),
        ListItem::new(""),
        doc_item("51-72", "Mathematical Modeling in Physics", "Kim, D. (2023)", "doi:10.1006/jmbi.2023.2354", "Kim2023", false),
        ListItem::new(""),
        doc_item("35-XX", "PDE Solutions for Nonlinear Systems", "Lee, S. · Park, M. (2023)", "arXiv:2301.00123", "Lee2023", true),
        ListItem::new(""),
        doc_item("53", "Quantum Hall Effect in Graphene", "Zhang, Y. (2024)", "doi:10.1103/PhysRevLett.132.046", "Zhang2024", false),
    ];

    let highlight = if focused {
        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::Black).fg(Color::DarkGray)
    };

    let list = List::default()
        .items(items)
        .style(Style::default().bg(Color::Black))
        .highlight_style(highlight);

    f.render_stateful_widget(list, area, state);
}

fn doc_item<'a>(code: &'a str, title: &'a str, author: &'a str, id: &'a str, key: &'a str, selected: bool) -> ListItem<'a> {
    let marker = if selected { "◆ " } else { "  " };
    let marker_color = if selected { Color::Yellow } else { Color::DarkGray };

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(marker, Style::default().fg(marker_color)),
            Span::styled(format!("{:<6} ", code), Style::default().fg(Color::Yellow)),
            Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("          "),
            Span::styled(author, Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(id, Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(format!("[{}]", key), Style::default().fg(Color::Green)),
        ]),
    ])
}

fn render_detail_panel(f: &mut Frame, area: Rect, focused: bool) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  제목   ", Style::default().fg(Color::Cyan)),
            Span::styled("PDE Solutions for Nonlinear Systems", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  저자   ", Style::default().fg(Color::Cyan)), Span::styled("Lee, S. · Park, M.", Style::default().fg(Color::Gray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  저널   ", Style::default().fg(Color::Cyan)), Span::styled("—", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  연도   ", Style::default().fg(Color::Cyan)), Span::styled("2023", Style::default().fg(Color::Gray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  DOI    ", Style::default().fg(Color::Cyan)), Span::styled("—", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  arXiv  ", Style::default().fg(Color::Cyan)), Span::styled("2301.00123", Style::default().fg(Color::Blue))]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  분류   ", Style::default().fg(Color::Cyan)),
            Span::styled("517.9 ", Style::default().fg(Color::Yellow)),
            Span::styled("미분방정식  ", Style::default().fg(Color::Gray)),
            Span::styled("(UDC)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("         "),
            Span::styled("35-XX ", Style::default().fg(Color::Green)),
            Span::styled("편미분방정식  ", Style::default().fg(Color::Gray)),
            Span::styled("(MSC)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  키     ", Style::default().fg(Color::Cyan)), Span::styled("Lee2023", Style::default().fg(Color::Green))]),
        Line::from(""),
        Line::from(vec![Span::styled("  파일   ", Style::default().fg(Color::Cyan)), Span::styled("~/.libran/library/Lee2023.pdf", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  출처   ", Style::default().fg(Color::Cyan)), Span::styled("PDF 자체 추출", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![Span::styled("  ─── 초록 ───", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![Span::raw("  " ), Span::styled("비선형 편미분방정식의 해법을 다룬 논문으로,", Style::default().fg(Color::Gray))]),
        Line::from(vec![Span::raw("  " ), Span::styled("분해법과 수치 해석을 결합한 접근을 제시한다.", Style::default().fg(Color::Gray))]),
        Line::from(vec![Span::raw("  " ), Span::styled("특히 반응-확산 계의 해 존재성을 증명하며...", Style::default().fg(Color::DarkGray))]),
    ];

    let style = if focused {
        Style::default().bg(Color::Black).fg(Color::White)
    } else {
        Style::default().bg(Color::Black).fg(Color::Gray)
    };

    let detail = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(style);

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
        spans.push(Span::styled(*key, Style::default().fg(Color::Cyan)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*label, Style::default().fg(Color::DarkGray)));
    }

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));
    f.render_widget(footer, area);
}

fn render_search_overlay(f: &mut Frame, area: Rect, input: &str) {
    let overlay_area = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: 3,
    };

    f.render_widget(Clear, overlay_area);

    let search_line = Line::from(vec![
        Span::raw(" "),
        Span::styled(">", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(input, Style::default().fg(Color::White)),
        Span::styled("▎", Style::default().fg(Color::Cyan)),
    ]);

    let search = Paragraph::new(search_line)
        .style(Style::default().bg(Color::Black));
    f.render_widget(search, overlay_area);
}

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(55, 65, area);
    f.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  단축키", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        help_line("  Tab", "패널 간 포커스 이동"),
        Line::from(""),
        help_line("  j / ↓", "아래로 이동"),
        Line::from(""),
        help_line("  k / ↑", "위로 이동"),
        Line::from(""),
        help_line("  Enter", "문헌 상세 보기 / 닫기"),
        Line::from(""),
        help_line("  /", "검색 모드 진입"),
        Line::from(""),
        help_line("  Space", "문헌 다중 선택 토글"),
        Line::from(""),
        help_line("  x", "BibTeX / CSL JSON 내보내기"),
        Line::from(""),
        help_line("  n", "새 프로젝트 생성"),
        Line::from(""),
        help_line("  ?", "도움말 토글"),
        Line::from(""),
        help_line("  q / Esc", "종료 (상세 보기 중일 시 상세 닫기)"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Drag & Drop", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("PDF 파일을 터미널로 드래그하여 추가", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ESC / ?", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .style(Style::default().bg(Color::Black));
    f.render_widget(help, popup_area);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(desc, Style::default().fg(Color::Gray)),
    ])
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
