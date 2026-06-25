use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::ui::theme;

pub const TOTAL_PAGES: usize = 3;

pub fn render(frame: &mut Frame, area: Rect, page: usize) {
    let popup = centered_rect(55, 80, area);
    frame.render_widget(Clear, popup);

    let lines = match page % TOTAL_PAGES {
        0 => page_0_lines(),
        1 => page_1_lines(),
        _ => page_2_lines(),
    };

    let help = Paragraph::new(lines)
        .style(Style::default().fg(theme::fg()).bg(theme::bg()))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, popup);
}

fn page_0_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        title_line("단축키"),
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
        help_line("  p", "외부 PDF 뷰어로 열기 (config viewer_command 있으면 그것, 없으면 시스템 기본)"),
        help_line("  u", "읽기 상태 토글: 안 읽음 → 읽는 중 → 읽음 → 안 읽음 (순환)"),
        help_line("  v", "현재 검색어를 이름과 함께 저장 (스마트 컬렉션)"),
        help_line("  i", "라이브러리 통계 대시보드 (i/Esc/q로 닫기)"),
        help_line("  w", "DOI/arXiv ID 일괄 가져오기 (한 줄에 하나, Enter 제출, 500ms 간격)"),
        help_line("  I", "BibTeX(.bib) 파일 가져오기 (경로 입력 → 자체 파서 → 일괄 등록)"),
        help_line("  F", "전방 인용 조회: 이 논문을 인용한 후속 논문 수 (OpenAlex, DOI 필요)"),
        help_line("  E", "저자 병합: 원본 저자명 → 정식 저자명 2단계 입력 (DB 일괄 교체)"),
        help_line("  b", "PDF 북마크/TOC 추출 (상세 보기에서, 페이지 번호와 함께 표시)"),
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
        help_line("  x", "내보내기 대화상자 (인용 복사 + 파일 내보내기)"),
        help_line("  o", "API 모드 토글"),
        help_line("  + / - / =", "사이드바 너비 조정 (+2 / -2 / 기본값)"),
        help_line("  q / Esc", "종료"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Drag & Drop",
                Style::default().fg(theme::selected()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "PDF 파일을 터미널로 드래그하여 추가",
                Style::default().fg(theme::fg()),
            ),
        ]),
        Line::from(""),
        page_footer(0),
    ]
}

fn page_1_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        title_line("내보내기 대화상자"),
        Line::from(""),
        sub_header("열기 · 닫기"),
        help_line("  x", "대화상자 열기 (문헌 선택 후)"),
        help_line("  Esc", "대화상자 닫기"),
        Line::from(""),
        sub_header("대화상자 조작"),
        help_line("  Tab / Shift+Tab", "섹션 이동 (형식→스타일→언어→표시→미리보기)"),
        help_line("  j / ↓", "항목 아래로 (미리보기 자동 업데이트)"),
        help_line("  k / ↑", "항목 위로"),
        help_line("  Enter", "클립보드에 인용 텍스트 복사 (선택 스타일)"),
        help_line("  e", "파일로 내보내기 (~/export.{확장자})"),
        Line::from(""),
        sub_header("참고"),
        help_line("  Enter", "스타일로 렌더링 → 클립보드 (논문 붙여넣기용)"),
        help_line("  e", "형식으로 파일 생성 → 참고문헌 관리 도구 가져오기"),
        help_line("  실패 시", "클립보드 불가 → ~/.libran/clipboard.txt 자동 저장"),
        help_line("  환경설정", "마지막 사용 조합 자동 저장 · 다음 복원"),
        Line::from(""),
        page_footer(1),
    ]
}

fn page_2_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        title_line("추가 기능 요약"),
        Line::from(""),
        help_line("  u", "읽기 상태 토글 (안 읽음 → 읽는 중 → 읽음, 리스트/상세 양쪽)"),
        help_line("  v", "현재 검색어를 이름과 함께 저장, 다시 불러와 적용 가능"),
        help_line("  i", "라이브러리 통계 대시보드 (총 문헌/읽기/연도/저자/저널)"),
        help_line("  b", "상세 보기에서 PDF 북마크/TOC 추출 (페이지 번호 포함)"),
        help_line("  w", "DOI/arXiv ID 여러 줄 일괄 가져오기 (CrossRef/arXiv API)"),
        help_line("  I", "BibTeX(.bib) 파일에서 문헌 일괄 등록"),
        help_line("  F", "전방 인용 조회 — 이 논문을 인용한 후속 논문 수 (OpenAlex, DOI 필요)"),
        help_line("  E", "저자 병합 — 원본 저자명을 정식 저자명으로 일괄 교체 (2단계 입력)"),
        Line::from(""),
        page_footer(2),
    ]
}

fn title_line(name: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {}", name),
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )])
}

fn page_footer(page: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {}/{}", page + 1, TOTAL_PAGES),
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "Tab 다음 페이지",
            Style::default().fg(theme::dim()),
        ),
        Span::raw("  "),
        Span::styled("Esc 닫기", Style::default().fg(theme::dim())),
    ])
}

fn sub_header(name: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  ▸ {}", name),
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::UNDERLINED),
    )])
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, Style::default().fg(theme::accent_primary())),
        Span::raw("  "),
        Span::styled(desc, Style::default().fg(theme::fg())),
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
