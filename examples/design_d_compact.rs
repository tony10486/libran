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

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| render(f, &mut list_state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
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

fn render(f: &mut Frame, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Libran ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" │ 머신러닝 가속 연구 │ "),
        Span::styled("517.9 미분방정식", Style::default().fg(Color::Yellow)),
        Span::raw(" │ "),
        Span::styled("오프라인", Style::default().fg(Color::Green)),
    ]));
    f.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(40)])
        .split(chunks[1]);

    let doc_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("517.9", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Nonisothermal Diffuse Interface Model", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("        Smith, J. (2024) · doi:10.33048/SIBJIM...")])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("51-72", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Mathematical Modeling in Physics", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("        Kim, D. (2023) · doi:10.1006/jmbi...")])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("▶ "),
            Span::styled("35-XX", Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("PDE Solutions for Nonlinear Systems", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("        Lee, S. · Park, M. (2023) · arXiv:2301.00123")])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("53   ", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Quantum Hall Effect in Graphene", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![Span::raw("        Zhang, Y. (2024) · doi:10.1103/PhysRevLett...")])),
    ];
    let doc_list = List::default()
        .items(doc_items)
        .block(Block::default().borders(Borders::LEFT))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(doc_list, body[0], list_state);

    let sidebar = Paragraph::new(vec![
        Line::from(vec![Span::styled(" 상세 정보", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))]),
        Line::from(""),
        Line::from(vec![Span::styled("제목  ", Style::default().fg(Color::Cyan)), Span::raw("PDE Solutions")]),
        Line::from(vec![Span::raw("      for Nonlinear Systems")]),
        Line::from(""),
        Line::from(vec![Span::styled("저자  ", Style::default().fg(Color::Cyan)), Span::raw("Lee, S.")]),
        Line::from(vec![Span::raw("      Park, M.")]),
        Line::from(""),
        Line::from(vec![Span::styled("연도  ", Style::default().fg(Color::Cyan)), Span::raw("2023")]),
        Line::from(""),
        Line::from(vec![Span::styled("DOI  ", Style::default().fg(Color::Cyan)), Span::raw("—")]),
        Line::from(""),
        Line::from(vec![Span::styled("ID   ", Style::default().fg(Color::Cyan)), Span::styled("arXiv:2301.00123", Style::default().fg(Color::Blue))]),
        Line::from(""),
        Line::from(vec![Span::styled("UDC  ", Style::default().fg(Color::Cyan)), Span::styled("517.9 ", Style::default().fg(Color::Yellow)), Span::raw("미분방정식")]),
        Line::from(vec![Span::styled("MSC  ", Style::default().fg(Color::Cyan)), Span::styled("35-XX ", Style::default().fg(Color::Green)), Span::raw("PDE")]),
        Line::from(""),
        Line::from(vec![Span::styled("키   ", Style::default().fg(Color::Cyan)), Span::styled("Lee2023", Style::default().fg(Color::Green))]),
        Line::from(""),
        Line::from(vec![Span::styled("파일 ", Style::default().fg(Color::Cyan)), Span::raw("Lee2023.pdf")]),
        Line::from(""),
        Line::from(vec![Span::styled("출처 ", Style::default().fg(Color::Cyan)), Span::raw("PDF 추출")]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(" 초록", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))]),
        Line::from(""),
        Line::from(" 비선형 편미분방정식의"),
        Line::from(" 해법을 다룬 논문으로,"),
        Line::from(" 분해법과 수치 해석을"),
        Line::from(" 결합한 접근을 제시..."),
    ])
    .wrap(Wrap { trim: false })
    .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(sidebar, body[1]);

    let bottom = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::Cyan)), Span::raw(" 이동 "),
        Span::styled(" /", Style::default().fg(Color::Cyan)), Span::raw(" 검색 "),
        Span::styled(" Space", Style::default().fg(Color::Cyan)), Span::raw(" 선택 "),
        Span::styled(" Tab", Style::default().fg(Color::Cyan)), Span::raw(" 분류 "),
        Span::styled(" x", Style::default().fg(Color::Cyan)), Span::raw(" 내보내기 "),
        Span::styled(" o", Style::default().fg(Color::Cyan)), Span::raw(" 온라인 "),
        Span::styled(" ?", Style::default().fg(Color::Cyan)), Span::raw(" 도움말 "),
        Span::styled(" q", Style::default().fg(Color::Cyan)), Span::raw(" 종료"),
    ]));
    f.render_widget(bottom, chunks[2]);
}
