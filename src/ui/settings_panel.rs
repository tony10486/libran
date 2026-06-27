use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::config::ColorConfig;
use crate::ui::theme;

/// 설정 패널 상태.
pub struct SettingsPanelState {
    pub cursor: usize,
    pub editing: bool,
    pub edit_input: String,
}

/// 설정 항목 인덱스.
const NUM_ITEMS: usize = 6;
const ITEM_BG_COLOR: usize = 0;
const ITEM_BG_FORCE: usize = 1;
const ITEM_ACCENT_PRIMARY: usize = 2;
const ITEM_ACCENT_SECONDARY: usize = 3;
const ITEM_SIDEBAR_WIDTH: usize = 4;
const ITEM_GLYPH_SET: usize = 5;

const ITEM_LABELS: [&str; NUM_ITEMS] = [
    "배경색",
    "배경 강제",
    "주 강조색",
    "보조 강조색",
    "사이드바 너비",
    "글리프 세트",
];

/// 설정 패널 렌더.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(65, 60, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_primary()))
        .title(Span::styled(
            " 설정 · Appearance ",
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // 내부를 preview + list + hint 로 분할
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // preview
            Constraint::Min(1),    // items
            Constraint::Length(1), // hint
        ])
        .split(inner);

    // ── 라이브 프리뷰 ──
    let preview = Paragraph::new(Line::from(vec![
        Span::styled(
            " Libran ",
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD)
                .bg(theme::bg()),
        ),
        Span::styled(
            " 문헌 관리 ",
            Style::default().fg(theme::accent_primary()).bg(theme::bg()),
        ),
        Span::styled(
            "● 읽음 ◐ 읽는중 ○ 안읽음",
            Style::default().fg(theme::selected()).bg(theme::bg()),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(preview, chunks[0]);

    // ── 항목 리스트 ──
    let lines = build_item_lines(state);
    let list = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(list, chunks[1]);

    // ── 도움말 ──
    let hint_text = if state.settings_panel.as_ref().map_or(false, |s| s.editing) {
        " #RRGGBB 입력 · Enter 확인 · Esc 취소 "
    } else {
        " j/k 탐색 · Enter 변경 · +/- 조정 · Esc 닫기 "
    };
    let hint = Paragraph::new(Span::styled(hint_text, Style::default().fg(theme::dim())))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[2]);
}

/// 각 항목의 라인 생성.
fn build_item_lines(state: &AppState) -> Vec<Line<'_>> {
    let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
    let editing = state.settings_panel.as_ref().map_or(false, |s| s.editing);
    let edit_input = state.settings_panel.as_ref().map_or("", |s| &s.edit_input);

    let mut lines: Vec<Line> = Vec::new();

    // 섹션 헤더
    lines.push(Line::from(Span::styled(
        " 외관 ",
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "──────────────",
        Style::default().fg(theme::divider()),
    )));

    for i in 0..NUM_ITEMS {
        let marker = if i == cursor { "▸ " } else { "  " };
        let label = ITEM_LABELS[i];
        let value = item_value_str(state, i, cursor, editing, edit_input);

        let label_style = if i == cursor {
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::fg())
        };
        let value_style = Style::default().fg(theme::meta());

        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(format!("{:<12}", label), label_style),
            Span::raw("  "),
            Span::styled(value, value_style),
        ]));
    }

    lines
}

/// 항목 값 문자열 반환.
fn item_value_str(
    state: &AppState,
    idx: usize,
    cursor: usize,
    editing: bool,
    edit_input: &str,
) -> String {
    // 편집 중인 항목이면 입력 버퍼 표시
    if editing && idx == cursor {
        return format!("{}█", edit_input);
    }

    match idx {
        ITEM_BG_COLOR => color_display(&state.config.theme.bg.color),
        ITEM_BG_FORCE => bool_display(state.config.theme.bg.force, "강제", "터미널 따름"),
        ITEM_ACCENT_PRIMARY => color_display(&state.config.theme.accent_primary),
        ITEM_ACCENT_SECONDARY => color_display(&state.config.theme.accent_secondary),
        ITEM_SIDEBAR_WIDTH => format!("{} 열", state.left_panel_width),
        ITEM_GLYPH_SET => state.glyph_set.clone(),
        _ => String::new(),
    }
}

/// 색상 표시 헬퍼.
fn color_display(opt: &Option<ColorConfig>) -> String {
    match opt {
        Some(cc) => match cc.to_rgb() {
            Some((r, g, b)) => format!("#{:02X}{:02X}{:02X}", r, g, b),
            None => "(잘못됨)".to_string(),
        },
        None => "기본값".to_string(),
    }
}

/// 불린 표시 헬퍼.
fn bool_display(val: bool, on: &str, off: &str) -> String {
    if val { on.to_string() } else { off.to_string() }
}

/// 설정 패널 키 처리. `true` 반환 = 종료 신호.
pub fn handle_key(state: &mut AppState, key: KeyEvent) -> bool {
    let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
    let editing = state.settings_panel.as_ref().map_or(false, |s| s.editing);

    if editing {
        return handle_edit_key(state, key, cursor);
    }

    match key.code {
        KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q') => {
            state.settings_panel_mode = false;
            state.settings_panel = None;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.cursor = (s.cursor + 1) % NUM_ITEMS;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.cursor = if s.cursor == 0 {
                    NUM_ITEMS - 1
                } else {
                    s.cursor - 1
                };
            }
        }
        KeyCode::Enter => handle_enter(state, cursor),
        KeyCode::Char('+') => {
            if cursor == ITEM_SIDEBAR_WIDTH {
                state.resize_left_panel(2);
            }
        }
        KeyCode::Char('-') => {
            if cursor == ITEM_SIDEBAR_WIDTH {
                state.resize_left_panel(-2);
            }
        }
        KeyCode::Char('=') => {
            if cursor == ITEM_SIDEBAR_WIDTH {
                state.reset_left_panel();
            }
        }
        _ => {}
    }
    false
}

/// Enter 처리: 항목 타입에 따라 토글/순환/편집 모드 진입.
fn handle_enter(state: &mut AppState, cursor: usize) {
    match cursor {
        ITEM_BG_FORCE => {
            state.config.theme.bg.force = !state.config.theme.bg.force;
            apply_and_save(state);
            state.set_status("배경 강제 토글됨");
        }
        ITEM_GLYPH_SET => {
            state.config.glyph_set = if state.config.glyph_set == "circles" {
                "ballot".to_string()
            } else {
                "circles".to_string()
            };
            state.glyph_set = state.config.glyph_set.clone();
            apply_and_save(state);
            state.set_status("글리프 세트 변경됨");
        }
        ITEM_BG_COLOR | ITEM_ACCENT_PRIMARY | ITEM_ACCENT_SECONDARY => {
            // 편집 모드 진입 — 현재 값을 초기값으로
            let current = match cursor {
                ITEM_BG_COLOR => state.config.theme.bg.color.clone(),
                ITEM_ACCENT_PRIMARY => state.config.theme.accent_primary.clone(),
                _ => state.config.theme.accent_secondary.clone(),
            };
            let init = match &current {
                Some(cc) => match cc.to_rgb() {
                    Some((r, g, b)) => format!("#{:02X}{:02X}{:02X}", r, g, b),
                    None => "#".to_string(),
                },
                None => "#".to_string(),
            };
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = true;
                s.edit_input = init;
            }
        }
        ITEM_SIDEBAR_WIDTH => {
            state.reset_left_panel();
            state.set_status("사이드바 너비 초기화됨");
        }
        _ => {}
    }
}

/// 편집 모드 키 처리.
fn handle_edit_key(state: &mut AppState, key: KeyEvent, cursor: usize) -> bool {
    match key.code {
        KeyCode::Esc => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = false;
                s.edit_input.clear();
            }
        }
        KeyCode::Enter => {
            if let Some(s) = state.settings_panel.as_ref() {
                let input = s.edit_input.clone();
                if is_valid_hex(&input) {
                    let cc = ColorConfig::Hex(input.clone());
                    match cursor {
                        ITEM_BG_COLOR => state.config.theme.bg.color = Some(cc),
                        ITEM_ACCENT_PRIMARY => state.config.theme.accent_primary = Some(cc),
                        ITEM_ACCENT_SECONDARY => state.config.theme.accent_secondary = Some(cc),
                        _ => {}
                    }
                    apply_and_save(state);
                    state.set_status("색상 변경됨");
                } else {
                    state.set_status("잘못된 색상 형식 (예: #94A3B8)");
                }
            }
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = false;
                s.edit_input.clear();
            }
        }
        KeyCode::Backspace => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.edit_input.pop();
            }
        }
        KeyCode::Char(c) => {
            // hex 문자 + # 만 허용
            if c.is_ascii_hexdigit() || c == '#' {
                if let Some(s) = state.settings_panel.as_mut() {
                    if s.edit_input.len() < 7 {
                        s.edit_input.push(c);
                    }
                }
            }
        }
        _ => {}
    }
    false
}

/// hex 문자열 검증 (#RRGGBB).
fn is_valid_hex(s: &str) -> bool {
    let s = s.strip_prefix('#').unwrap_or(s);
    s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// config 저장 + 테마 재적용.
fn apply_and_save(state: &AppState) {
    let _ = state.config.save();
    theme::init_theme(theme::Theme::from_config(&state.config.theme));
}

/// 중앙 팝업 영역 계산.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let pop_w = area.width * percent_x / 100;
    let pop_h = area.height * percent_y / 100;
    let x = area.x + (area.width.saturating_sub(pop_w)) / 2;
    let y = area.y + (area.height.saturating_sub(pop_h)) / 2;
    Rect::new(x, y, pop_w, pop_h)
}
