use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut left_state = ListState::default();
    left_state.select(Some(0));
    let mut right_state = ListState::default();
    right_state.select(Some(0));
    let mut active_panel = 0;

    loop {
        terminal.draw(|f| render(f, &mut left_state, &mut right_state, active_panel))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Tab => active_panel = 1 - active_panel,
                KeyCode::Down | KeyCode::Char('j') => {
                    if active_panel == 0 {
                        left_state.select_next();
                    } else {
                        right_state.select_next();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if active_panel == 0 {
                        left_state.select_previous();
                    } else {
                        right_state.select_previous();
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

fn render(
    f: &mut Frame,
    left_state: &mut ListState,
    right_state: &mut ListState,
    active_panel: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[0]);

    let left_block_style = if active_panel == 0 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let left_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![Span::styled(" 프로젝트", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))])),
        ListItem::new("   머신러닝 가속 연구"),
        ListItem::new("   CUI 렌더러 설계"),
        ListItem::new("   수학적 모델링"),
        ListItem::new(""),
        ListItem::new(Line::from(vec![Span::styled(" 분류 (UDC)", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))])),
        ListItem::new(" ▸ 0  총류 · 정보학"),
        ListItem::new(" ▸ 1  철학 · 심리학"),
        ListItem::new(" ▸ 2  종교"),
        ListItem::new(" ▸ 3  사회과학"),
        ListItem::new(" ▸ 5  자연과학            (12)"),
        ListItem::new("   ▸ 51 수학              (5)"),
        ListItem::new("     ▸ 517 해석학          (3)"),
        ListItem::new("       517.9 미분방정식     (2)"),
        ListItem::new("   ▸ 53 물리학            (4)"),
        ListItem::new(" ▸ 6  응용과학            (7)"),
        ListItem::new(" ▸ 7  예술"),
        ListItem::new(" ▸ 8  언어"),
        ListItem::new(" ▸ 9  역사"),
        ListItem::new(""),
        ListItem::new(" 분류 (PhySH)"),
        ListItem::new("   ▸ Condensed Matter     (3)"),
        ListItem::new("   ▸ Quantum Information  (2)"),
    ];

    let left_list = List::default()
        .items(left_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Libran ")
                .border_style(left_block_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(left_list, body[0], left_state);

    let right_block_style = if active_panel == 1 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let right_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("517.9", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Nonisothermal Diffuse Interface Model", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("Smith, J. (2024)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("doi:10.33048/SIBJIM.2022.25.103", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("▶ "),
            Span::styled("51-72", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("Mathematical Modeling in Physics", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("Kim, D. (2023)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("doi:10.1006/jmbi.2023.2354", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("35-XX", Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("PDE Solutions for Nonlinear Systems", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("Lee, S. · Park, M. (2023)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("arXiv:2301.00123", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("53", Style::default().fg(Color::Yellow)),
            Span::raw("   "),
            Span::styled("Quantum Hall Effect in Graphene", Style::default().add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("Zhang, Y. (2024)", Style::default().fg(Color::Gray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("         "),
            Span::styled("doi:10.1103/PhysRevLett.132.046", Style::default().fg(Color::DarkGray)),
        ])),
    ];

    let right_list = List::default()
        .items(right_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 문헌 (4) ")
                .border_style(right_block_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(right_list, body[1], right_state);

    let status = Paragraph::new(Line::from(vec![
        Span::raw(" 준비됨"),
        Span::raw(" │ API: "),
        Span::styled("오프라인", Style::default().fg(Color::Green)),
        Span::raw(" │ 4 문헌 │ "),
        Span::styled("Libran v0.1", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(status, chunks[1]);
}
