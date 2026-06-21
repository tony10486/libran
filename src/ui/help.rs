use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(55, 80, area);
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
        sub_header("네비게이션"),
        help_line("  Tab", "패널 간 포커스 이동"),
        help_line("  j / ↓", "아래로 이동"),
        help_line("  k / ↑", "위로 이동"),
        help_line("  Enter", "문헌 상세 보기 / 닫기"),
        help_line("  ←", "상세 보기 닫기"),
        help_line("  ?", "도움말 토글"),
        Line::from(""),
        sub_header("문헌 관리"),
        help_line("  /", "검색 모드 진입"),
        help_line("  a", "파일 경로 입력 추가"),
        help_line("  Space", "문헌 다중 선택 토글"),
        help_line("  e", "문헌 메타데이터 편집"),
        help_line("  t", "태그 편집 (상세 보기에서)"),
        help_line("  r", "별점 설정 (상세 보기에서, 1-5)"),
        help_line("  s", "유사도 순 정렬 (선택 문헌 기준)"),
        help_line("  Esc", "유사도 정렬 해제"),
        help_line("  d", "문헌 삭제 (확인 대화상자)"),
        help_line("  D", "CrossRef로 메타데이터 재조회 (DOI/arXiv 필요)"),
        help_line("  0-9", "UDC 분류 번호로 바로 선택 (왼쪽 패널)"),
        help_line("  c", "커스텀 필드 추가 (상세 보기에서)"),
        Line::from(""),
        sub_header("프로젝트 · 시리즈"),
        help_line("  n", "새 프로젝트 생성"),
        help_line("  m", "선택 문헌을 프로젝트에 추가 (다이얼로그)"),
        help_line("  S", "새 시리즈 생성"),
        help_line("  M", "시리즈 그룹핑 토글"),
        help_line("  A", "같은 저널 문헌 자동 묶기"),
        help_line("  Enter", "왼쪽 패널에서 연구자 섹션 펼치기/접기 + 선택"),
        help_line("  f", "연구자 이름 검색 (섹션 펼친 후)"),
        help_line("  H", "연구자 지표 조회 (h-index, i10-index)"),
        help_line("  K", "OpenAlex API 키 등록"),
        help_line("  B", "자동 지표 조회 토글 (저자 선택 시 자동, 7일 캐시)"),
        Line::from(""),
        sub_header("내보내기 · 설정"),
        help_line("  x", "BibTeX 내보내기"),
        help_line("  o", "API 모드 토글"),
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
        .style(Style::default().fg(Color::Gray).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, popup);
}

fn sub_header(name: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  ▸ {}", name),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
    )])
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
