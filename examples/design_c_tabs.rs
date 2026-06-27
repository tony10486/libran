use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut active_tab = 0;

    loop {
        terminal.draw(|f| render(f, &mut list_state, active_tab))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('1') => active_tab = 0,
                KeyCode::Char('2') => active_tab = 1,
                KeyCode::Char('3') => active_tab = 2,
                KeyCode::Char('4') => active_tab = 3,
                KeyCode::Down | KeyCode::Char('j') => list_state.select_next(),
                KeyCode::Up | KeyCode::Char('k') => list_state.select_previous(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(f: &mut Frame, list_state: &mut ListState, active_tab: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "  Libran ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ "),
        Span::styled("오프라인", Style::default().fg(Color::Green)),
        Span::raw(" │ 4 문헌"),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let tab_titles = vec![" 문헌 ", " 프로젝트 ", " 분류 ", " 설정 "];
    let tabs = Tabs::new(
        tab_titles
            .iter()
            .map(|t| Line::from(*t))
            .collect::<Vec<_>>(),
    )
    .select(active_tab)
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );
    f.render_widget(tabs, chunks[0]);

    match active_tab {
        0 => render_documents_tab(f, chunks[1], list_state),
        1 => render_projects_tab(f, chunks[1], list_state),
        2 => render_classification_tab(f, chunks[1], list_state),
        _ => render_settings_tab(f, chunks[1]),
    }

    let bottom = Paragraph::new(Line::from(vec![
        Span::styled(" 1-4", Style::default().fg(Color::Cyan)),
        Span::raw(" 탭 "),
        Span::styled(" j/k", Style::default().fg(Color::Cyan)),
        Span::raw(" 이동 "),
        Span::styled(" /", Style::default().fg(Color::Cyan)),
        Span::raw(" 검색 "),
        Span::styled(" Space", Style::default().fg(Color::Cyan)),
        Span::raw(" 선택 "),
        Span::styled(" x", Style::default().fg(Color::Cyan)),
        Span::raw(" 내보내기 "),
        Span::styled(" q", Style::default().fg(Color::Cyan)),
        Span::raw(" 종료  "),
    ]));
    f.render_widget(bottom, chunks[2]);
}

fn render_documents_tab(f: &mut Frame, area: ratatui::layout::Rect, list_state: &mut ListState) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("517.9  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Nonisothermal Diffuse Interface Model",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Smith, J. (2024)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled(
                "doi:10.33048/SIBJIM.2022.25.103",
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled("[Smith2024]", Style::default().fg(Color::Green)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("▶ "),
            Span::styled("51-72  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Mathematical Modeling in Physics",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Kim, D. (2023)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled(
                "doi:10.1006/jmbi.2023.2354",
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled("[Kim2023]", Style::default().fg(Color::Green)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("35-XX  ", Style::default().fg(Color::Green)),
            Span::styled(
                "PDE Solutions for Nonlinear Systems",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Lee, S. (2023)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("arXiv:2301.00123", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("[Lee2023]", Style::default().fg(Color::Green)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("53     ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Quantum Hall Effect in Graphene",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Zhang, Y. (2024)", Style::default().fg(Color::Gray)),
        ])),
    ];
    let list = List::default()
        .items(items)
        .block(Block::default().borders(Borders::ALL).title(" 문헌 (4) "))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, list_state);
}

fn render_projects_tab(f: &mut Frame, area: ratatui::layout::Rect, list_state: &mut ListState) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                "  머신러닝 가속 연구  ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("(12)", Style::default().fg(Color::Yellow)),
        ])),
        ListItem::new("    2024-01 ~ 현재 · 다학제 연구"),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  CUI 렌더러 설계  ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("(7)", Style::default().fg(Color::Yellow)),
        ])),
        ListItem::new("    2024-03 ~ 현재 · 컴퓨터과학"),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "  수학적 모델링  ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("(5)", Style::default().fg(Color::Yellow)),
        ])),
        ListItem::new("    2023-09 ~ 2024-02 · 수학·물리학"),
        ListItem::new(""),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  [n]", Style::default().fg(Color::Cyan)),
            Span::raw(" 새 프로젝트 만들기"),
        ])),
    ];
    let list = List::default()
        .items(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 프로젝트 (3) "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, list_state);
}

fn render_classification_tab(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    list_state: &mut ListState,
) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                " UDC",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" (메인) "),
            Span::styled("CC BY-SA 3.0", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new("  ▸ 0  총류 · 정보학"),
        ListItem::new("  ▸ 1  철학 · 심리학"),
        ListItem::new("  ▸ 5  자연과학              (12)"),
        ListItem::new("    ▸ 51 수학                 (5)"),
        ListItem::new("      ▸ 517 해석학            (3)"),
        ListItem::new("        517.9 미분방정식       (2)"),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                " PhySH",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" (보조) "),
            Span::styled("CC0", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new("  ▸ Condensed Matter          (3)"),
        ListItem::new("  ▸ Quantum Information       (2)"),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                " MSC2020",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" (보조) "),
            Span::styled("CC BY-NC-SA", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new("  ▸ 35 편미분방정식            (1)"),
    ];
    let list = List::default()
        .items(items)
        .block(Block::default().borders(Borders::ALL).title(" 분류 체계 "))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, list_state);
}

fn render_settings_tab(f: &mut Frame, area: ratatui::layout::Rect) {
    let settings = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  API 모드        ", Style::default().fg(Color::Cyan)),
            Span::raw("오프라인"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  파일 보관       ", Style::default().fg(Color::Cyan)),
            Span::raw("라이브러리 복사"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  인용키 모드     ", Style::default().fg(Color::Cyan)),
            Span::raw("성+연도 (AuthorYear)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  메인 분류       ", Style::default().fg(Color::Cyan)),
            Span::raw("UDC Summary"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  활성 스킴       ", Style::default().fg(Color::Cyan)),
            Span::raw("UDC, PhySH, MSC2020"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  라벨 언어       ", Style::default().fg(Color::Cyan)),
            Span::raw("English"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  DB 경로        ", Style::default().fg(Color::Cyan)),
            Span::raw("~/.libran/libran.db"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [e]", Style::default().fg(Color::Yellow)),
            Span::raw(" 편집  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" 확인"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" 설정 "));
    f.render_widget(settings, area);
}
