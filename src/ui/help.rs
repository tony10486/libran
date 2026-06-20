use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(55, 70, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  단축키",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
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
        help_line("  a", "파일 경로 입력 추가"),
        Line::from(""),
        help_line("  Space", "문헌 다중 선택 토글"),
        Line::from(""),
        help_line("  e", "문헌 메타데이터 편집"),
        Line::from(""),
        help_line("  s", "유사도 순 정렬 (선택 문헌 기준)"),
        Line::from(""),
        help_line("  Esc", "유사도 정렬 해제"),
        Line::from(""),
        help_line("  d", "문헌 삭제"),
        Line::from(""),
        help_line("  x", "BibTeX 내보내기"),
        Line::from(""),
        help_line("  n", "새 프로젝트 생성"),
        Line::from(""),
        help_line("  o", "API 모드 토글"),
        Line::from(""),
        help_line("  ?", "도움말 토글"),
        Line::from(""),
        help_line("  q / Esc", "종료"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Drag & Drop", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("PDF 파일을 터미널로 드래그하여 추가", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Esc / ?", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("도움말 닫기", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let help = Paragraph::new(lines)
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, popup);
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
