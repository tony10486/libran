use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tree_state = ListState::default();
    tree_state.select(Some(0));
    let mut doc_state = ListState::default();
    doc_state.select(Some(0));
    let mut focus = 0;

    loop {
        terminal.draw(|f| render(f, &mut tree_state, &mut doc_state, focus))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Tab => focus = (focus + 1) % 3,
                KeyCode::Down | KeyCode::Char('j') => {
                    match focus {
                        0 => tree_state.select_next(),
                        1 => doc_state.select_next(),
                        _ => {}
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    match focus {
                        0 => tree_state.select_previous(),
                        1 => doc_state.select_previous(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(f: &mut Frame, tree_state: &mut ListState, doc_state: &mut ListState, focus: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let top_bar = Paragraph::new(Line::from(vec![
        Span::styled(" Libran ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        Span::raw(" │ "),
        Span::raw("머신러닝 가속 연구"),
        Span::raw(" │ "),
        Span::styled("4 문헌", Style::default().fg(Color::Yellow)),
        Span::raw(" │ "),
        Span::styled("오프라인", Style::default().fg(Color::Green)),
    ]));
    f.render_widget(top_bar, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(40), Constraint::Percentage(35)])
        .split(chunks[1]);

    let tree_style = if focus == 0 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) };
    let tree_items: Vec<ListItem> = vec![
        ListItem::new("프로젝트"),
        ListItem::new("  ML 가속"),
        ListItem::new("  CUI 렌더러"),
        ListItem::new(""),
        ListItem::new("UDC 분류"),
        ListItem::new("  ▸ 0 총류"),
        ListItem::new("  ▾ 5 자연과학 (12)"),
        ListItem::new("    ▾ 51 수학 (5)"),
        ListItem::new("      ▸ 512 대수"),
        ListItem::new("      ▾ 517 해석학 (3)"),
        ListItem::new("        ▸ 517.9 미분방정식"),
        ListItem::new("    ▸ 53 물리학 (4)"),
        ListItem::new("  ▸ 6 응용과학 (7)"),
        ListItem::new(""),
        ListItem::new("PhySH"),
        ListItem::new("  ▸ 응축물질 (3)"),
    ];
    let tree_list = List::default()
        .items(tree_items)
        .block(Block::default().borders(Borders::ALL).title(" 탐색 ").border_style(tree_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(tree_list, body[0], tree_state);

    let doc_style = if focus == 1 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) };
    let doc_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  517.9 ", Style::default().fg(Color::Yellow)),
            Span::styled("Nonisothermal Diffuse Interface", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("         Smith, J. (2024)")])) ,
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  51-72 ", Style::default().fg(Color::Yellow)),
            Span::styled("Mathematical Modeling in Physics", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("         Kim, D. (2023)")])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  35-XX ", Style::default().fg(Color::Green)),
            Span::styled("PDE Solutions for Nonlinear", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("         Lee, S. · Park, M. (2023)")])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("  53    ", Style::default().fg(Color::Yellow)),
            Span::styled("Quantum Hall Effect in Graphene", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("         Zhang, Y. (2024)")])),
    ];
    let doc_list = List::default()
        .items(doc_items)
        .block(Block::default().borders(Borders::ALL).title(" 문헌 ").border_style(doc_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(doc_list, body[1], doc_state);

    let detail_style = if focus == 2 { Style::default().fg(Color::Cyan) } else { Style::default().fg(Color::DarkGray) };
    let detail = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("제목   ", Style::default().fg(Color::Cyan)),
            Span::raw("Nonisothermal Diffuse Interface"),
        ]),
        Line::from(vec![
            Span::raw("        Model for Electrical Breakdown"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("저자   ", Style::default().fg(Color::Cyan)), Span::raw("Smith, J. · Kim, D.")]),
        Line::from(""),
        Line::from(vec![Span::styled("저널   ", Style::default().fg(Color::Cyan)), Span::raw("SIBJIM (2022)")]),
        Line::from(""),
        Line::from(vec![Span::styled("DOI    ", Style::default().fg(Color::Cyan)), Span::styled("10.33048/SIBJIM.2022.25.103", Style::default().fg(Color::Blue))]),
        Line::from(""),
        Line::from(vec![Span::styled("분류   ", Style::default().fg(Color::Cyan)), Span::styled("517.9 ", Style::default().fg(Color::Yellow)), Span::raw("미분방정식")]),
        Line::from(vec![Span::raw("       "), Span::styled("51-72 ", Style::default().fg(Color::Yellow)), Span::raw("수학적 모델링·물리학")]),
        Line::from(""),
        Line::from(vec![Span::styled("키     ", Style::default().fg(Color::Cyan)), Span::styled("Smith2024", Style::default().fg(Color::Green))]),
        Line::from(""),
        Line::from(vec![Span::styled("파일   ", Style::default().fg(Color::Cyan)), Span::raw("~/.libran/library/Smith2024.pdf")]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled("초록", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))]),
        Line::from(""),
        Line::from("  본 연구는 전기 브레이크다운 채널 전파에"),
        Line::from("  대한 비등온 확산 인터페이스 모델을 제시"),
        Line::from("  한다. 이 모델은 온도 의존적 전기 전도도를"),
        Line::from("  가진 매질에서의 채널 형성을 설명하며..."),
    ])
    .wrap(Wrap { trim: false })
    .block(Block::default().borders(Borders::ALL).title(" 상세 ").border_style(detail_style));
    f.render_widget(detail, body[2]);

    let bottom = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::Cyan)), Span::raw(" 이동 "),
        Span::styled(" j/k", Style::default().fg(Color::Cyan)), Span::raw(" 탐색 "),
        Span::styled(" /", Style::default().fg(Color::Cyan)), Span::raw(" 검색 "),
        Span::styled(" Space", Style::default().fg(Color::Cyan)), Span::raw(" 선택 "),
        Span::styled(" x", Style::default().fg(Color::Cyan)), Span::raw(" 내보내기 "),
        Span::styled(" q", Style::default().fg(Color::Cyan)), Span::raw(" 종료"),
    ]));
    f.render_widget(bottom, chunks[2]);
}
