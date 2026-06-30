use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::config::ColorConfig;
use crate::ui::theme;

// ── Tab 상수 ───────────────────────────────────────────────────────────────────

pub const TAB_MAIN: usize = 0;
pub const TAB_THEME: usize = 1;
const NUM_TABS: usize = 2;

const TAB_LABELS: [&str; NUM_TABS] = [" 기본 설정 ", " 테마 색상 "];

/// 설정 패널 상태.
pub struct SettingsPanelState {
    /// 활성화된 탭: 0 = 기본 설정, 1 = 테마 색상
    pub active_tab: usize,
    pub cursor: usize,
    pub editing: bool,
    pub edit_input: String,
    pub theme_dropdown_open: bool,
    pub available_themes: Vec<String>,
    pub theme_cursor: usize,
    pub color_picker_open: bool,
    pub picker_grid_x: usize,
    pub picker_grid_y: usize,
    pub picker_hue: f32,
    /// 테마 색상 탭 내 커서.
    pub theme_color_cursor: usize,
}

// ── 팔레트 상수 ──────────────────────────────────────────────────────────────────

const PALETTE_ROWS: usize = 10;
const PALETTE_COLS: usize = 24;

// ── HSV 변환 ────────────────────────────────────────────────────────────────────

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r_prime + m) * 255.0).round() as u8,
        ((g_prime + m) * 255.0).round() as u8,
        ((b_prime + m) * 255.0).round() as u8,
    )
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h, s, v)
}

// ── 메인 메뉴 항목 상수 ─────────────────────────────────────────────────────────

const MAIN_NUM_ITEMS: usize = 8;
const MAIN_THEME: usize = 0;
const MAIN_BG_FORCE: usize = 1;
const MAIN_SIDEBAR_WIDTH: usize = 2;
const MAIN_GLYPH_SET: usize = 3;
const MAIN_PDF_VIEWER: usize = 4;
const MAIN_WIDGET_PANEL: usize = 5;
const MAIN_IMPORT_DB: usize = 6;
const MAIN_RESET: usize = 7;

const MAIN_LABELS: [&str; MAIN_NUM_ITEMS] = [
    "테마 이름",
    "배경 강제",
    "사이드바 너비",
    "글리프 세트",
    "PDF 뷰어",
    "▶ 위젯 패널 열기",
    "▶ DB 불러오기",
    "기본값으로 초기화",
];


// ── 테마 색상 탭 상수 ────────────────────────────────────────────────────────────

const TC_NUM_ITEMS: usize = 20;
const TC_BG: usize = 0;
const TC_SURFACE: usize = 1;
const TC_FG: usize = 2;
const TC_ACCENT_PRIMARY: usize = 3;
const TC_ACCENT_SECONDARY: usize = 4;
const TC_DIM: usize = 5;
const TC_DIVIDER: usize = 6;
const TC_SELECTED: usize = 7;
const TC_FOCUS_BG: usize = 8;
const TC_FOCUS_FG: usize = 9;
const TC_TITLE: usize = 10;
const TC_META: usize = 11;
const TC_KEY: usize = 12;
const TC_TAG: usize = 13;
const TC_UDC: usize = 14;
const TC_ERROR: usize = 15;
const TC_WARNING: usize = 16;
const TC_CODE: usize = 17;
const TC_SUCCESS: usize = 18;
const TC_RESET: usize = 19;
// search_bg는 고급 사용자용이므로 탭에서 제외 (config 상에는 존재)

const TC_LABELS: [&str; TC_NUM_ITEMS] = [
    "배경색",
    "표면색",
    "전경색",
    "주 강조색",
    "보조 강조색",
    "흐림색",
    "구분선색",
    "선택색",
    "포커스 배경",
    "포커스 전경",
    "제목색",
    "메타색",
    "키색",
    "태그색",
    "UDC색",
    "오류색",
    "경고색",
    "코드색",
    "성공색",
    "── 기본값으로 초기화 ──",
];

// ── Render ──────────────────────────────────────────────────────────────────────

/// 설정 패널 렌더.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let active_tab = state
        .settings_panel
        .as_ref()
        .map_or(0, |s| s.active_tab);

    let title = "설정 · Settings";

    // 테마 색상 탭은 항목이 많으므로 팝업을 약간 더 크게
    let (pop_w, pop_h) = if active_tab == TAB_THEME {
        (70, 72)
    } else {
        (65, 60)
    };
    let popup = centered_rect(pop_w, pop_h, area);
    frame.render_widget(Clear, popup);

    let block = theme::create_theme_block(title);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let preview_height: u16 = if active_tab == TAB_THEME { 0 } else { 3 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                 // tab bar
            Constraint::Length(preview_height),    // preview or nothing
            Constraint::Min(1),                    // items
            Constraint::Length(1),                 // hint
        ])
        .split(inner);

    // ── 탭 바 ──
    render_tab_bar(frame, chunks[0], active_tab);

    // ── 라이브 프리뷰 (기본 설정 탭에만 표시) ──
    if active_tab != TAB_THEME {
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
        frame.render_widget(preview, chunks[1]);
    }

    // ── 항목 리스트 ──
    let lines = build_item_lines(state);
    let list = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(list, chunks[2]);

    // ── 도움말 ──
    let hint_text = if active_tab == TAB_THEME {
        if state.settings_panel.as_ref().map_or(false, |s| s.color_picker_open) {
            " ↑/↓ 채널 선택 · ◀/▶ 값 가감 · Enter 적용 · Esc 취소 "
        } else {
            " ↑/↓ 탐색 · Enter 색상 편집 · Tab 탭 전환 · Esc/p 닫기 "
        }
    } else if state.settings_panel.as_ref().map_or(false, |s| s.editing) {
        if state.settings_panel.as_ref().map_or(false, |s| s.color_picker_open) {
            " ↑/↓ 채널 선택 · ◀/▶ 값 가감 · Enter 적용 · Esc 취소 "
        } else {
            " #RRGGBB 입력 · Enter 확인 · Esc 취소 "
        }
    } else {
        " ↑/↓ 탐색 · Enter 변경 · +/- 너비 조정 · Tab 탭 전환 · Esc/p 닫기 "
    };
    let hint = Paragraph::new(Span::styled(hint_text, Style::default().fg(theme::dim())))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);

    // ── 드롭다운 / 컬러 피커 ──
    if let Some(s) = state.settings_panel.as_ref() {
        if s.theme_dropdown_open {
            render_theme_dropdown(frame, s, area);
        } else if s.color_picker_open {
            render_color_picker(frame, s, area);
        }
    }
}

// ── 탭 바 렌더 ─────────────────────────────────────────────────────────────────

fn render_tab_bar(frame: &mut Frame, area: Rect, active_tab: usize) {
    let mut spans = Vec::new();
    for i in 0..NUM_TABS {
        let label = TAB_LABELS[i];
        if i == active_tab {
            spans.push(Span::styled(
                format!("[{}]", label),
                Style::default()
                    .fg(theme::accent_primary())
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(theme::dim()),
            ));
        }
    }
    let tab_bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(tab_bar, area);
}

// ── 드롭다운 렌더 ───────────────────────────────────────────────────────────────

fn render_theme_dropdown(frame: &mut Frame, s: &SettingsPanelState, area: Rect) {
    let select_box_area = centered_rect(45, 45, area);
    frame.render_widget(Clear, select_box_area);

    let select_block = theme::create_theme_block(" 테마 선택 · Select Theme ");
    let select_inner = select_block.inner(select_box_area);
    frame.render_widget(select_block, select_box_area);

    let mut list_lines = Vec::new();
    for (idx, theme_name) in s.available_themes.iter().enumerate() {
        let is_current = idx == s.theme_cursor;
        let marker = if is_current { " ▶ " } else { "   " };

        let style = if is_current {
            theme::focus_style()
        } else if theme_name == &crate::config::AppConfig::default().theme_name {
            theme::selected_style()
        } else {
            Style::default().fg(theme::fg())
        };

        list_lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(theme_name.clone(), style),
        ]));
    }

    let list_widget = Paragraph::new(list_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme::bg()));
    frame.render_widget(list_widget, select_inner);
}

// ── 컬러 피커 렌더 ──────────────────────────────────────────────────────────────

fn render_color_picker(frame: &mut Frame, s: &SettingsPanelState, area: Rect) {
    let picker_area = centered_rect(72, 64, area);
    frame.render_widget(Clear, picker_area);

    static PICKER_TITLE: &str = " 🎨 True Color 그라데이션 팔레트 · Color Picker ";
    let picker_block = theme::create_theme_block(PICKER_TITLE);
    let picker_inner = picker_block.inner(picker_area);

    // 블록을 먼저 렌더링 (배경색 + 테두리)
    frame.render_widget(picker_block, picker_area);

    // picker_inner를 수직으로: S-V grid + Hue bar + Hue marker + preview
    let picker_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(PALETTE_ROWS as u16),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(picker_inner);

    let grid_area = picker_chunks[0];
    let hue_bar_area = picker_chunks[1];
    let hue_marker_area = picker_chunks[2];
    let preview_area = picker_chunks[3];

    // ── 2D S-V 그라데이션 격자 ──
    let mut grid_lines = Vec::new();
    for r in 0..PALETTE_ROWS {
        let mut spans = Vec::new();
        spans.push(Span::raw("   "));
        for c in 0..PALETTE_COLS {
            let s_val = c as f32 / 23.0;
            let v_val = 1.0 - (r as f32 / 9.0);

            let (red, green, blue) = hsv_to_rgb(s.picker_hue, s_val, v_val);
            let color = ratatui::style::Color::Rgb(red, green, blue);

            let is_focused = r == s.picker_grid_y && c == s.picker_grid_x;
            if is_focused {
                let yuv =
                    0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32;
                let circle_color = if yuv < 128.0 {
                    ratatui::style::Color::White
                } else {
                    ratatui::style::Color::Black
                };
                spans.push(Span::styled(
                    "●●",
                    Style::default().fg(circle_color).bg(color),
                ));
            } else {
                spans.push(Span::styled(
                    "██",
                    Style::default().fg(color).bg(color),
                ));
            }
        }
        grid_lines.push(Line::from(spans));
    }
    let grid_widget = Paragraph::new(grid_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme::bg()));
    frame.render_widget(grid_widget, grid_area);

    // ── Hue 막대 (S-V 그리드 색상 영역과 정렬: 48칸) ──
    let hue_bar_width = 48u16;
    let hue_bar_x = grid_area.x + 3;
    let mut hue_spans = Vec::new();
    for col in 0..hue_bar_width {
        let hue = (col as f32 / (hue_bar_width - 1) as f32) * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        let c = ratatui::style::Color::Rgb(r, g, b);
        hue_spans.push(Span::styled("█", Style::default().fg(c).bg(c)));
    }
    let hue_bar_rect = Rect::new(hue_bar_x, hue_bar_area.y, hue_bar_width, 1);
    frame.render_widget(
        Paragraph::new(Line::from(hue_spans))
            .wrap(Wrap { trim: false }),
        hue_bar_rect,
    );

    // ── Hue 마커 (▲) ──
    let marker_pos = ((s.picker_hue / 360.0) * ((hue_bar_width - 1) as f32)).round() as u16;
    let mut marker_spans = Vec::new();
    for col in 0..hue_bar_width {
        if col == marker_pos {
            marker_spans.push(Span::styled(
                "▲",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            marker_spans.push(Span::raw(" "));
        }
    }
    let hue_marker_rect = Rect::new(hue_bar_x, hue_marker_area.y, hue_bar_width, 1);
    frame.render_widget(
        Paragraph::new(Line::from(marker_spans))
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(theme::bg())),
        hue_marker_rect,
    );

    // ── 하단 미리보기 ──
    let s_val = s.picker_grid_x as f32 / 23.0;
    let v_val = 1.0 - (s.picker_grid_y as f32 / 9.0);
    let (red, green, blue) = hsv_to_rgb(s.picker_hue, s_val, v_val);
    let hex = format!("#{:02X}{:02X}{:02X}", red, green, blue);
    let preview_color = ratatui::style::Color::Rgb(red, green, blue);

    let preview_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("   선택한 색상:  "),
            Span::styled(
                hex,
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   색조(Hue): "),
            Span::styled(
                format!("{:.0}°", s.picker_hue),
                Style::default().fg(theme::accent_primary()),
            ),
            Span::raw("  채도(Sat): "),
            Span::styled(
                format!("{}%", (s_val * 100.0) as usize),
                Style::default().fg(theme::fg()),
            ),
            Span::raw("  명도(Val): "),
            Span::styled(
                format!("{}%", (v_val * 100.0) as usize),
                Style::default().fg(theme::fg()),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("   [ 미리보기 ]  "),
            Span::styled(
                "                                                  ",
                Style::default().bg(preview_color),
            ),
        ]),
    ];

    let preview_widget = Paragraph::new(preview_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(theme::bg()));
    frame.render_widget(preview_widget, preview_area);
}

// ── 항목 라인 생성 ──────────────────────────────────────────────────────────────

fn build_item_lines(state: &AppState) -> Vec<Line<'_>> {
    let active_tab = state
        .settings_panel
        .as_ref()
        .map_or(0, |s| s.active_tab);
    if active_tab == TAB_THEME {
        return build_theme_color_lines(state);
    }
    build_main_lines(state)
}

fn build_main_lines(state: &AppState) -> Vec<Line<'_>> {
    let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);

    let mut lines: Vec<Line> = Vec::new();

    // ── 외관 섹션 ─────────────────────────────────────────
    lines.push(Line::from(Span::styled(
        " 외관 · Appearance ",
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "────────────────────────",
        Style::default().fg(theme::divider()),
    )));

    for i in [MAIN_THEME, MAIN_BG_FORCE, MAIN_SIDEBAR_WIDTH, MAIN_GLYPH_SET, MAIN_PDF_VIEWER] {
        let marker = if i == cursor { "▸ " } else { "  " };
        let label = MAIN_LABELS[i];
        let value = item_value_str(state, i);

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
            Span::styled(format!("{:<14}", label), label_style),
            Span::raw("  "),
            Span::styled(value, value_style),
        ]));
    }

    // ── 도구 섹션 ─────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::raw(""),
    ]));
    lines.push(Line::from(Span::styled(
        " 도구 · Tools ",
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "────────────────────────",
        Style::default().fg(theme::divider()),
    )));

    for i in [MAIN_WIDGET_PANEL, MAIN_IMPORT_DB] {
        let marker = if i == cursor { "▸ " } else { "  " };
        let label = MAIN_LABELS[i];

        let label_style = if i == cursor {
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::fg())
        };

        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(format!("{:<14}", label), label_style),
        ]));
    }

    // ── 초기화 섹션 ───────────────────────────────────────
    lines.push(Line::from(vec![
        Span::raw(""),
    ]));
    lines.push(Line::from(Span::styled(
        "────────────────────────",
        Style::default().fg(theme::divider()),
    )));

    for i in [MAIN_RESET] {
        let marker = if i == cursor { "▸ " } else { "  " };
        let label = MAIN_LABELS[i];

        let label_style = if i == cursor {
            Style::default()
                .fg(theme::error())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::error())
        };

        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(format!("{:<14}", label), label_style),
        ]));
    }

    lines
}

fn build_theme_color_lines(state: &AppState) -> Vec<Line<'_>> {
    let cursor = state
        .settings_panel
        .as_ref()
        .map_or(0, |s| s.theme_color_cursor);
    let editing = state.settings_panel.as_ref().map_or(false, |s| s.editing);
    let edit_input = state
        .settings_panel
        .as_ref()
        .map_or("", |s| &s.edit_input);

    let mut lines: Vec<Line> = Vec::new();

    // 헤더: 각 항목에서 Enter → 컬러 피커 또는 #RRGGBB 직접 입력
    lines.push(Line::from(Span::styled(
        " 각 항목에서 Enter → 컬러 피커 또는 #RRGGBB 직접 입력 ",
        Style::default().fg(theme::dim()),
    )));

    for i in 0..TC_NUM_ITEMS {
        let marker = if i == cursor { "▸ " } else { "  " };
        let label = TC_LABELS[i];

        if i == TC_RESET {
            // 구분선 + 초기화 항목
            let label_style = if i == cursor {
                Style::default()
                    .fg(theme::error())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::error())
            };
            lines.push(Line::from(vec![
                Span::raw(marker),
                Span::styled(format!("{:<14}", label), label_style),
            ]));
            continue;
        }

        let value = if editing && i == cursor {
            format!("{}█", edit_input)
        } else {
            match get_theme_color(&state.config.theme, i) {
                Some(cc) => match cc.to_rgb() {
                    Some((r, g, b)) => format!("#{:02X}{:02X}{:02X}", r, g, b),
                    None => "(잘못됨)".to_string(),
                },
                None => "기본값".to_string(),
            }
        };

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
            Span::styled(format!("{:<14}", label), label_style),
            Span::raw("  "),
            Span::styled(value, value_style),
        ]));
    }

    lines
}

// ── 항목 값 문자열 ──────────────────────────────────────────────────────────────

fn item_value_str(state: &AppState, idx: usize) -> String {
    let editing = state.settings_panel.as_ref().map_or(false, |s| s.editing);
    let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
    let edit_input = state
        .settings_panel
        .as_ref()
        .map_or("", |s| &s.edit_input);

    if editing && idx == cursor {
        return format!("{}█", edit_input);
    }

    match idx {
        MAIN_THEME => state.config.theme_name.clone(),
        MAIN_BG_FORCE => bool_display(state.config.theme.bg.force, "강제", "터미널 따름"),
        MAIN_SIDEBAR_WIDTH => format!("{} 열", state.left_panel_width),
        MAIN_GLYPH_SET => state.glyph_set.clone(),
        MAIN_PDF_VIEWER => {
            match &state.config.viewer_command {
                Some(parts) if !parts.is_empty() => {
                    if parts.len() >= 3
                        && parts[0] == "open"
                        && parts[1] == "-a"
                        && parts[2] == "Preview"
                    {
                        "macOS Preview (미리보기)".to_string()
                    } else if parts[0] == "okular" {
                        "Okular (자동 동기화)".to_string()
                    } else {
                        format!("커스텀: {}", parts.join(" "))
                    }
                }
                _ => "Sioyek (기본값 / 자동 동기화)".to_string(),
            }
        }
        _ => String::new(),
    }
}

/// theme config에서 색상값을 읽는다.
fn get_theme_color(theme: &crate::config::ThemeConfig, idx: usize) -> Option<ColorConfig> {
    match idx {
        TC_BG => theme.bg.color.clone(),
        TC_SURFACE => theme.surface.clone(),
        TC_FG => theme.fg.clone(),
        TC_ACCENT_PRIMARY => theme.accent_primary.clone(),
        TC_ACCENT_SECONDARY => theme.accent_secondary.clone(),
        TC_DIM => theme.dim.clone(),
        TC_DIVIDER => theme.divider.clone(),
        TC_SELECTED => theme.selected.clone(),
        TC_FOCUS_BG => theme.focus_bg.clone(),
        TC_FOCUS_FG => theme.focus_fg.clone(),
        TC_TITLE => theme.title.clone(),
        TC_META => theme.meta.clone(),
        TC_KEY => theme.key.clone(),
        TC_TAG => theme.tag.clone(),
        TC_UDC => theme.udc.clone(),
        TC_ERROR => theme.error.clone(),
        TC_WARNING => theme.warning.clone(),
        TC_CODE => theme.code.clone(),
        TC_SUCCESS => theme.success.clone(),
        _ => None,
    }
}

/// theme config에 색상값을 쓴다.
fn set_theme_color(theme: &mut crate::config::ThemeConfig, idx: usize, color: Option<ColorConfig>) {
    match idx {
        TC_BG => theme.bg.color = color,
        TC_SURFACE => theme.surface = color,
        TC_FG => theme.fg = color,
        TC_ACCENT_PRIMARY => theme.accent_primary = color,
        TC_ACCENT_SECONDARY => theme.accent_secondary = color,
        TC_DIM => theme.dim = color,
        TC_DIVIDER => theme.divider = color,
        TC_SELECTED => theme.selected = color,
        TC_FOCUS_BG => theme.focus_bg = color,
        TC_FOCUS_FG => theme.focus_fg = color,
        TC_TITLE => theme.title = color,
        TC_META => theme.meta = color,
        TC_KEY => theme.key = color,
        TC_TAG => theme.tag = color,
        TC_UDC => theme.udc = color,
        TC_ERROR => theme.error = color,
        TC_WARNING => theme.warning = color,
        TC_CODE => theme.code = color,
        TC_SUCCESS => theme.success = color,
        _ => {}
    }
}

// ── 헬퍼 ────────────────────────────────────────────────────────────────────────

fn bool_display(val: bool, on: &str, off: &str) -> String {
    if val { on.to_string() } else { off.to_string() }
}

/// config 저장 + 테마 재적용.
fn apply_and_save(state: &AppState) {
    let _ = state.config.save();
    theme::init_theme(theme::load_theme(&state.config));
}

/// 중앙 팝업 영역 계산.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let pop_w = area.width * percent_x / 100;
    let pop_h = area.height * percent_y / 100;
    let x = area.x + (area.width.saturating_sub(pop_w)) / 2;
    let y = area.y + (area.height.saturating_sub(pop_h)) / 2;
    Rect::new(x, y, pop_w, pop_h)
}

/// hex 문자열 검증 (#RRGGBB).
fn is_valid_hex(s: &str) -> bool {
    let s = s.strip_prefix('#').unwrap_or(s);
    s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit())
}

// ── Key Handling ────────────────────────────────────────────────────────────────

/// 설정 패널 키 처리. `true` 반환 = 종료 신호.
pub fn handle_key(state: &mut AppState, key: KeyEvent) -> bool {
    let active_tab = state.settings_panel.as_ref().map_or(0, |s| s.active_tab);
    let color_picker_open = state
        .settings_panel
        .as_ref()
        .map_or(false, |s| s.color_picker_open);
    let theme_dropdown_open = state
        .settings_panel
        .as_ref()
        .map_or(false, |s| s.theme_dropdown_open);
    let editing = state.settings_panel.as_ref().map_or(false, |s| s.editing);

    if color_picker_open {
        return handle_picker_key(state, key);
    }

    if theme_dropdown_open {
        return handle_dropdown_key(state, key);
    }

    if editing && active_tab == TAB_THEME {
        let cursor = state
            .settings_panel
            .as_ref()
            .map_or(0, |s| s.theme_color_cursor);
        return handle_theme_color_edit_key(state, key, cursor);
    }

    if editing && active_tab == TAB_MAIN {
        let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
        return handle_edit_key(state, key, cursor);
    }

    // ── Tab 키로 탭 전환 ──
    if key.code == KeyCode::Tab {
        if let Some(s) = state.settings_panel.as_mut() {
            s.active_tab = (s.active_tab + 1) % NUM_TABS;
            s.cursor = 0;
            s.theme_color_cursor = 0;
            s.editing = false;
            s.edit_input.clear();
            s.color_picker_open = false;
        }
        state.dirty = true;
        return false;
    }

    if active_tab == TAB_THEME {
        return handle_theme_color_key(state, key);
    }

    // ── TAB_MAIN: 기본 설정 ──
    match key.code {
        KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('q') => {
            state.settings_panel_mode = false;
            state.settings_panel = None;
        }
        KeyCode::Down => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.cursor = (s.cursor + 1) % MAIN_NUM_ITEMS;
            }
        }
        KeyCode::Up => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.cursor = if s.cursor == 0 {
                    MAIN_NUM_ITEMS - 1
                } else {
                    s.cursor - 1
                };
            }
        }
        KeyCode::Enter => handle_main_enter(state),
        KeyCode::Char('+') => {
            let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
            if cursor == MAIN_SIDEBAR_WIDTH {
                state.resize_left_panel(2);
            }
        }
        KeyCode::Char('-') => {
            let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
            if cursor == MAIN_SIDEBAR_WIDTH {
                state.resize_left_panel(-2);
            }
        }
        KeyCode::Char('=') => {
            let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
            if cursor == MAIN_SIDEBAR_WIDTH {
                state.reset_left_panel();
            }
        }
        _ => {}
    }
    state.dirty = true;
    false
}

// ── 메인 메뉴 Enter 처리 ────────────────────────────────────────────────────────

fn handle_main_enter(state: &mut AppState) {
    let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);

    match cursor {
        MAIN_THEME => {
            let available = theme::get_available_themes();
            let current = state.config.theme_name.clone();
            let cur_idx = available.iter().position(|r| r == &current).unwrap_or(0);
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = true;
                s.theme_dropdown_open = true;
                s.available_themes = available;
                s.theme_cursor = cur_idx;
            }
        }
        MAIN_BG_FORCE => {
            state.config.theme.bg.force = !state.config.theme.bg.force;
            apply_and_save(state);
            state.set_status("배경 강제 토글됨");
        }
        MAIN_SIDEBAR_WIDTH => {
            state.reset_left_panel();
            state.set_status("사이드바 너비 초기화됨");
        }
        MAIN_GLYPH_SET => {
            state.config.glyph_set = if state.config.glyph_set == "circles" {
                "ballot".to_string()
            } else {
                "circles".to_string()
            };
            state.glyph_set = state.config.glyph_set.clone();
            apply_and_save(state);
            state.set_status("글리프 세트 변경됨");
        }
        MAIN_PDF_VIEWER => {
            let mode = match &state.config.viewer_command {
                None => "sioyek",
                Some(parts) if parts.is_empty() => "sioyek",
                Some(parts) => {
                    if parts.len() >= 3
                        && parts[0] == "open"
                        && parts[1] == "-a"
                        && parts[2] == "Preview"
                    {
                        "preview"
                    } else if parts[0] == "okular" {
                        "okular"
                    } else {
                        "custom"
                    }
                }
            };

            match mode {
                "sioyek" => {
                    state.config.viewer_command = Some(vec![
                        "okular".to_string(),
                        "%p".to_string(),
                    ]);
                    apply_and_save(state);
                    state.set_status("PDF 뷰어: Okular (자동 동기화)");
                }
                "okular" => {
                    state.config.viewer_command = Some(vec![
                        "open".to_string(),
                        "-a".to_string(),
                        "Preview".to_string(),
                        "%p".to_string(),
                    ]);
                    apply_and_save(state);
                    state.set_status("PDF 뷰어: macOS Preview (미리보기)");
                }
                _ => {
                    state.config.viewer_command = None;
                    apply_and_save(state);
                    state.set_status("PDF 뷰어: Sioyek (기본값 / 자동 감지)");
                }
            }
        }
        MAIN_WIDGET_PANEL => {
            // 위젯 패널 열기 (설정 패널 닫음)
            state.settings_panel_mode = false;
            state.settings_panel = None;
            state.show_widget_panel = true;
        }
        MAIN_IMPORT_DB => {
            // DB 불러오기: 설정 패널을 닫고 명령 모드로 진입
            state.settings_panel_mode = false;
            state.settings_panel = None;
            state.command_mode = true;
            state.command_input = ":import-db ".to_string();
            state.set_status("가져올 DB 파일 경로 입력");
        }
        MAIN_RESET => {
            // 기본 설정 탭 초기화: ThemeConfig의 기본값으로 복원
            state.config.theme_name = crate::config::AppConfig::default().theme_name;
            state.config.theme.bg.force = crate::config::AppConfig::default().theme.bg.force;
            state.config.glyph_set = crate::config::AppConfig::default().glyph_set;
            state.glyph_set = state.config.glyph_set.clone();
            state.config.viewer_command = None;
            state.left_panel_width = 28;
            apply_and_save(state);
            state.set_status("기본 설정이 초기화되었습니다.");
        }
        _ => {}
    }
}

// ── 테마 색상 서브메뉴 키 처리 ─────────────────────────────────────────────────

fn handle_theme_color_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('q') => {
            state.settings_panel_mode = false;
            state.settings_panel = None;
        }
        KeyCode::Down => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.theme_color_cursor = (s.theme_color_cursor + 1) % TC_NUM_ITEMS;
            }
        }
        KeyCode::Up => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.theme_color_cursor = if s.theme_color_cursor == 0 {
                    TC_NUM_ITEMS - 1
                } else {
                    s.theme_color_cursor - 1
                };
            }
        }
        KeyCode::Enter => {
            let idx = state
                .settings_panel
                .as_ref()
                .map_or(0, |s| s.theme_color_cursor);
            if idx == TC_RESET {
                // 테마 색상을 None(기본값)으로 초기화
                state.config.theme = crate::config::ThemeConfig::default();
                apply_and_save(state);
                state.set_status("테마 색상이 기본값으로 초기화되었습니다.");
            } else {
                open_color_picker(state, idx);
            }
        }
        _ => {}
    }
    state.dirty = true;
    false
}

/// 컬러 피커 열기. `idx`는 TC_* 인덱스 (서브메뉴 모드).
fn open_color_picker(state: &mut AppState, idx: usize) {
    let current = get_theme_color(&state.config.theme, idx);

    let (r, g, b) = match &current {
        Some(cc) => cc.to_rgb().unwrap_or((128, 128, 128)),
        None => match idx {
            TC_BG => match theme::bg() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (22, 22, 22),
            },
            TC_SURFACE => match theme::surface() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (28, 28, 28),
            },
            TC_FG => match theme::fg() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (200, 200, 200),
            },
            TC_ACCENT_PRIMARY => match theme::accent_primary() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (107, 155, 210),
            },
            TC_ACCENT_SECONDARY => match theme::accent_secondary() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (184, 169, 212),
            },
            TC_DIM => match theme::dim() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (128, 128, 128),
            },
            TC_DIVIDER => match theme::divider() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (50, 50, 50),
            },
            TC_SELECTED => match theme::selected() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (126, 184, 138),
            },
            TC_FOCUS_BG => match theme::focus_bg() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (38, 38, 38),
            },
            TC_FOCUS_FG => match theme::focus_fg() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (200, 200, 200),
            },
            TC_TITLE => match theme::fg() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (200, 200, 200),
            },
            TC_META => match theme::meta() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (128, 128, 128),
            },
            TC_KEY => match theme::key() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (126, 184, 138),
            },
            TC_TAG => match theme::tag() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (184, 169, 212),
            },
            TC_UDC => match theme::udc() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (107, 155, 210),
            },
            TC_ERROR => match theme::error() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (200, 140, 150),
            },
            TC_WARNING => match theme::warning() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (212, 166, 87),
            },
            TC_CODE => match theme::code() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (107, 155, 210),
            },
            TC_SUCCESS => match theme::success() {
                ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
                _ => (126, 184, 138),
            },
            _ => (128, 128, 128),
        },
    };

    let (h, s_val, v_val) = rgb_to_hsv(r, g, b);
    let grid_x = ((s_val * 23.0).round() as usize).min(23);
    let grid_y = (((1.0 - v_val) * 9.0).round() as usize).min(9);

    if let Some(panel) = state.settings_panel.as_mut() {
        panel.editing = true;
        panel.color_picker_open = true;
        panel.picker_grid_x = grid_x;
        panel.picker_grid_y = grid_y;
        panel.picker_hue = h;
    }
}

// ── 컬러 피커 키 처리 ──────────────────────────────────────────────────────────

fn handle_picker_key(state: &mut AppState, key: KeyEvent) -> bool {
    let mut picker_done = false;
    let mut picker_cancel = false;
    let mut chosen_hex: Option<String> = None;

    if let Some(s) = state.settings_panel.as_mut() {
        match key.code {
            KeyCode::Esc => {
                picker_cancel = true;
            }
            KeyCode::Up => {
                s.picker_grid_y = if s.picker_grid_y == 0 {
                    PALETTE_ROWS - 1
                } else {
                    s.picker_grid_y - 1
                };
            }
            KeyCode::Down => {
                s.picker_grid_y = (s.picker_grid_y + 1) % PALETTE_ROWS;
            }
            KeyCode::Left => {
                s.picker_grid_x = if s.picker_grid_x == 0 {
                    PALETTE_COLS - 1
                } else {
                    s.picker_grid_x - 1
                };
            }
            KeyCode::Right => {
                s.picker_grid_x = (s.picker_grid_x + 1) % PALETTE_COLS;
            }
            KeyCode::Enter => {
                let s_val = s.picker_grid_x as f32 / 23.0;
                let v_val = 1.0 - (s.picker_grid_y as f32 / 9.0);
                let (r, g, b) = hsv_to_rgb(s.picker_hue, s_val, v_val);
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                chosen_hex = Some(hex);
                picker_done = true;
            }
            _ => {}
        }
    }

    if let Some(hex) = chosen_hex {
        let is_theme_tab = state
            .settings_panel
            .as_ref()
            .is_some_and(|s| s.active_tab == TAB_THEME);
        let cc = ColorConfig::Hex(hex);

        if is_theme_tab {
            // 테마 탭: TC_* 인덱스 사용
            let tc_idx = state
                .settings_panel
                .as_ref()
                .map_or(0, |s| s.theme_color_cursor);
            set_theme_color(&mut state.config.theme, tc_idx, Some(cc));
        } else {
            let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
            match cursor {
                MAIN_BG_FORCE => {}
                _ => {}
            }
        }
        apply_and_save(state);
        state.set_status("색상이 변경되었습니다.");
    }

    if picker_done || picker_cancel {
        if let Some(s) = state.settings_panel.as_mut() {
            s.color_picker_open = false;
            s.editing = false;
        }
    }

    state.dirty = true;
    false
}

// ── 드롭다운 키 처리 ────────────────────────────────────────────────────────────

fn handle_dropdown_key(state: &mut AppState, key: KeyEvent) -> bool {
    let mut theme_to_save: Option<String> = None;
    let mut close_dropdown = false;

    if let Some(s) = state.settings_panel.as_mut() {
        let total = s.available_themes.len();
        if total == 0 {
            s.theme_dropdown_open = false;
            s.editing = false;
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                s.theme_dropdown_open = false;
                s.editing = false;
            }
            KeyCode::Down => {
                s.theme_cursor = (s.theme_cursor + 1) % total;
            }
            KeyCode::Up => {
                s.theme_cursor = if s.theme_cursor == 0 {
                    total - 1
                } else {
                    s.theme_cursor - 1
                };
            }
            KeyCode::Enter => {
                let chosen = s.available_themes[s.theme_cursor].clone();
                theme_to_save = Some(chosen);
                close_dropdown = true;
            }
            _ => {}
        }
    }

    if let Some(chosen) = theme_to_save {
        state.config.theme_name = chosen;
        apply_and_save(state);
        state.set_status("테마가 변경되었습니다.");
    }

    if close_dropdown {
        if let Some(s) = state.settings_panel.as_mut() {
            s.theme_dropdown_open = false;
            s.editing = false;
        }
    }

    state.dirty = true;
    false
}

// ── 편집 모드 키 처리 (메인 메뉴) ───────────────────────────────────────────────

fn handle_edit_key(state: &mut AppState, key: KeyEvent, _cursor: usize) -> bool {
    match key.code {
        KeyCode::Esc => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = false;
                s.edit_input.clear();
            }
        }
        KeyCode::Enter => {
            if let Some(s) = state.settings_panel.as_ref() {
                let input = s.edit_input.trim().to_string();
                if is_valid_hex(&input) {
                    let _cc = ColorConfig::Hex(input.clone());

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
    state.dirty = true;
    false
}

// ── 편집 모드 키 처리 (테마 색상 서브메뉴) ──────────────────────────────────────

fn handle_theme_color_edit_key(state: &mut AppState, key: KeyEvent, cursor: usize) -> bool {
    match key.code {
        KeyCode::Esc => {
            if let Some(s) = state.settings_panel.as_mut() {
                s.editing = false;
                s.edit_input.clear();
            }
        }
        KeyCode::Enter => {
            if let Some(s) = state.settings_panel.as_ref() {
                let input = s.edit_input.trim().to_string();
                if is_valid_hex(&input) {
                    let cc = ColorConfig::Hex(input.clone());
                    set_theme_color(&mut state.config.theme, cursor, Some(cc));
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
    state.dirty = true;
    false
}

// ── Mouse Handling ──────────────────────────────────────────────────────────────

pub fn handle_mouse_click(state: &mut AppState, column: u16, row: u16) -> bool {
    let (w, h) = crossterm::terminal::size().unwrap_or(state.terminal_size);
    state.terminal_size = (w, h);
    let area = Rect::new(0, 0, w, h);

    let mut chosen_hex: Option<String> = None;
    let mut click_consumed = false;

    if let Some(s) = state.settings_panel.as_mut() {
        if s.color_picker_open {
            click_consumed = true;
            let picker_area = centered_rect(72, 64, area);
            let picker_block = theme::create_theme_block(
                " 🎨 True Color 그라데이션 팔레트 · Color Picker ",
            );
            let picker_inner = picker_block.inner(picker_area);

            // render_color_picker와 동일한 레이아웃
            let picker_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(PALETTE_ROWS as u16),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ])
                .split(picker_inner);

            let grid_area = picker_chunks[0];
            let hue_bar_area = picker_chunks[1];
            let hue_marker_area = picker_chunks[2];

            let grid_start_x = grid_area.x + 3;
            let hue_bar_x = grid_start_x;
            let hue_bar_width = 48u16;

            let in_grid = column >= grid_start_x && column < grid_start_x + 48
                && row >= grid_area.y
                && row < grid_area.y + PALETTE_ROWS as u16;

            let in_hue_bar = column >= hue_bar_x
                && column < hue_bar_x + hue_bar_width
                && (row == hue_bar_area.y || row == hue_marker_area.y);

            if in_grid {
                let c = ((column - grid_start_x) / 2) as usize;
                let r = (row - grid_area.y) as usize;
                if c < PALETTE_COLS && r < PALETTE_ROWS {
                    s.picker_grid_x = c;
                    s.picker_grid_y = r;
                    let s_val = c as f32 / 23.0;
                    let v_val = 1.0 - (r as f32 / 9.0);
                    let (red, green, blue) = hsv_to_rgb(s.picker_hue, s_val, v_val);
                    chosen_hex =
                        Some(format!("#{:02X}{:02X}{:02X}", red, green, blue));
                }
            } else if in_hue_bar {
                let c = (column - hue_bar_x) as usize;
                let hue = (c as f32 / ((hue_bar_width - 1) as f32)) * 360.0;
                s.picker_hue = hue;
            }
        }
    }

    if let Some(hex) = chosen_hex {
        let is_theme_tab = state
            .settings_panel
            .as_ref()
            .is_some_and(|s| s.active_tab == TAB_THEME);
        let cc = ColorConfig::Hex(hex);

        if is_theme_tab {
            let tc_idx = state
                .settings_panel
                .as_ref()
                .map_or(0, |s| s.theme_color_cursor);
            set_theme_color(&mut state.config.theme, tc_idx, Some(cc));
        } else {
            let cursor = state.settings_panel.as_ref().map_or(0, |s| s.cursor);
            match cursor {
                MAIN_BG_FORCE => {}
                _ => {}
            }
        }
        apply_and_save(state);
        state.set_status("색상이 변경되었습니다.");

        if let Some(s) = state.settings_panel.as_mut() {
            s.color_picker_open = false;
            s.editing = false;
        }
    }

    if click_consumed {
        state.dirty = true;
    }
    click_consumed
}

pub fn handle_mouse_hover(state: &mut AppState, column: u16, row: u16) -> bool {
    let (w, h) = crossterm::terminal::size().unwrap_or(state.terminal_size);
    state.terminal_size = (w, h);
    let area = Rect::new(0, 0, w, h);

    let mut hover_consumed = false;

    if let Some(s) = state.settings_panel.as_mut() {
        if s.color_picker_open {
            let picker_area = centered_rect(72, 64, area);
            let picker_block = theme::create_theme_block(
                " 🎨 True Color 그라데이션 팔레트 · Color Picker ",
            );
            let picker_inner = picker_block.inner(picker_area);

            // render_color_picker와 동일한 레이아웃
            let picker_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(PALETTE_ROWS as u16),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ])
                .split(picker_inner);

            let grid_area = picker_chunks[0];
            let hue_bar_area = picker_chunks[1];
            let hue_marker_area = picker_chunks[2];

            let grid_start_x = grid_area.x + 3;
            let hue_bar_x = grid_start_x;
            let hue_bar_width = 48u16;

            if column >= grid_start_x && column < grid_start_x + 48
                && row >= grid_area.y
                && row < grid_area.y + PALETTE_ROWS as u16
            {
                let c = ((column - grid_start_x) / 2) as usize;
                let r = (row - grid_area.y) as usize;
                if c < PALETTE_COLS && r < PALETTE_ROWS {
                    s.picker_grid_x = c;
                    s.picker_grid_y = r;
                    hover_consumed = true;
                }
            }

            if column >= hue_bar_x
                && column < hue_bar_x + hue_bar_width
                && (row == hue_bar_area.y || row == hue_marker_area.y)
            {
                let c = (column - hue_bar_x) as usize;
                let target_hue =
                    (c as f32 / ((hue_bar_width - 1) as f32)) * 360.0;
                s.picker_hue = target_hue;
                hover_consumed = true;
            }
        }
    }

    if hover_consumed {
        state.dirty = true;
    }
    hover_consumed
}
