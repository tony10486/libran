use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, BorderType};
use std::env;
use std::fs;
use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::config::{ColorConfig, ThemeConfig};

// ── Theme struct ──────────────────────────────────────────────

/// Central theme with 21 semantic color slots.
/// All colors are stored as `Color::Rgb` on truecolor terminals,
/// or `Color::Indexed` (quantized) on 256-color terminals.
/// Named colors (`Color::Cyan`, `Color::Black`, etc.) are never used
/// because terminals can remap them, defeating forced backgrounds.
#[derive(Clone)]
pub struct Theme {
    pub bg: Color,
    /// Slightly lighter than bg — used for left panel and overlays.
    pub surface: Color,
    pub fg: Color,
    /// Blue RGB(107,155,210) — used for headers, labels, UDC codes, structure.
    pub accent_primary: Color,
    /// Lavender RGB(184,169,212) — used for brand/active emphasis and tags.
    pub accent_secondary: Color,
    pub dim: Color,
    pub divider: Color,
    pub selected: Color,
    pub focus_bg: Color,
    pub focus_fg: Color,
    pub title: Color,
    pub meta: Color,
    pub key: Color,
    pub tag: Color,
    pub udc: Color,
    pub error: Color,
    pub warning: Color,
    pub code: Color,
    pub success: Color,
    pub search_bg: Color,
    /// When true, `.bg(theme::bg())` returns the configured bg color.
    /// When false, returns `Color::Reset` (terminal default bg).
    pub force_bg: bool,

    // ── 신규 비주얼 커스텀 파라미터 ──
    pub sidebar_right: bool,
    pub search_bottom: bool,
    pub title_bold: bool,
    pub title_underline: bool,
    pub header_bold: bool,
    pub header_underline: bool,

    // 테두리 스타일
    pub border_type: String,
    pub show_border_top: bool,
    pub show_border_bottom: bool,
    pub show_border_left: bool,
    pub show_border_right: bool,

    // 글리프 및 진행 바
    pub active_pointer: String,
    pub bullet_marker: String,
    pub radio_checked: String,
    pub radio_unchecked: String,
    pub progress_filled: String,
    pub progress_unfilled: String,

    // 스크롤바
    pub scrollbar_thumb: String,
    pub scrollbar_track: String,

    // 읽음 상태
    pub unread_marker: String,
    pub unread_color: Color,
    pub reading_marker: String,
    pub reading_color: Color,
    pub read_marker: String,
    pub read_color: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self::dark()
    }

    pub fn dark() -> Self {
        let tc = detect_truecolor();
        let rgb = |r: u8, g: u8, b: u8| -> Color {
            if tc {
                Color::Rgb(r, g, b)
            } else {
                quantize_to_256(r, g, b)
            }
        };
        Theme {
            bg: rgb(22, 22, 22),
            surface: rgb(28, 28, 28),
            fg: rgb(200, 200, 200),
            accent_primary: rgb(107, 155, 210),
            accent_secondary: rgb(184, 169, 212),
            dim: rgb(128, 128, 128),
            divider: rgb(50, 50, 50),
            selected: rgb(126, 184, 138),
            focus_bg: rgb(38, 38, 38),
            focus_fg: rgb(200, 200, 200),
            title: rgb(200, 200, 200),
            meta: rgb(128, 128, 128),
            key: rgb(126, 184, 138),
            tag: rgb(184, 169, 212),
            udc: rgb(107, 155, 210),
            error: rgb(200, 140, 150),
            warning: rgb(212, 166, 87),
            code: rgb(107, 155, 210),
            success: rgb(126, 184, 138),
            search_bg: rgb(28, 28, 28),
            force_bg: true,

            sidebar_right: false,
            search_bottom: false,
            title_bold: true,
            title_underline: false,
            header_bold: true,
            header_underline: false,

            border_type: "plain".to_string(),
            show_border_top: true,
            show_border_bottom: true,
            show_border_left: true,
            show_border_right: true,

            active_pointer: "►".to_string(),
            bullet_marker: "▸".to_string(),
            radio_checked: "●".to_string(),
            radio_unchecked: "○".to_string(),
            progress_filled: "━".to_string(),
            progress_unfilled: "─".to_string(),

            scrollbar_thumb: "█".to_string(),
            scrollbar_track: "░".to_string(),

            unread_marker: "○".to_string(),
            unread_color: rgb(128, 128, 128),
            reading_marker: "◐".to_string(),
            reading_color: rgb(212, 166, 87),
            read_marker: "●".to_string(),
            read_color: rgb(126, 184, 138),
        }
    }

    pub fn light() -> Self {
        let tc = detect_truecolor();
        let rgb = |r: u8, g: u8, b: u8| -> Color {
            if tc {
                Color::Rgb(r, g, b)
            } else {
                quantize_to_256(r, g, b)
            }
        };
        Theme {
            bg: rgb(245, 245, 245),
            surface: rgb(235, 235, 235),
            fg: rgb(40, 40, 40),
            accent_primary: rgb(30, 80, 150),
            accent_secondary: rgb(100, 50, 150),
            dim: rgb(120, 120, 120),
            divider: rgb(200, 200, 200),
            selected: rgb(40, 120, 60),
            focus_bg: rgb(220, 220, 220),
            focus_fg: rgb(20, 20, 20),
            title: rgb(40, 40, 40),
            meta: rgb(110, 110, 110),
            key: rgb(40, 120, 60),
            tag: rgb(100, 50, 150),
            udc: rgb(30, 80, 150),
            error: rgb(180, 40, 40),
            warning: rgb(180, 100, 20),
            code: rgb(30, 80, 150),
            success: rgb(40, 120, 60),
            search_bg: rgb(230, 230, 230),
            force_bg: true,

            sidebar_right: false,
            search_bottom: false,
            title_bold: true,
            title_underline: false,
            header_bold: true,
            header_underline: false,

            border_type: "plain".to_string(),
            show_border_top: true,
            show_border_bottom: true,
            show_border_left: true,
            show_border_right: true,

            active_pointer: "►".to_string(),
            bullet_marker: "▸".to_string(),
            radio_checked: "●".to_string(),
            radio_unchecked: "○".to_string(),
            progress_filled: "━".to_string(),
            progress_unfilled: "─".to_string(),

            scrollbar_thumb: "█".to_string(),
            scrollbar_track: "░".to_string(),

            unread_marker: "○".to_string(),
            unread_color: rgb(120, 120, 120),
            reading_marker: "◐".to_string(),
            reading_color: rgb(180, 100, 20),
            read_marker: "●".to_string(),
            read_color: rgb(40, 120, 60),
        }
    }

    /// Returns the configured bg color if `force_bg` is true,
    /// otherwise `Color::Reset` (terminal default).
    pub fn bg_color(&self) -> Color {
        if self.force_bg { self.bg } else { Color::Reset }
    }

    /// Build a Theme from ThemeConfig, filling unset slots with defaults.
    /// Invalid hex strings fall back to the default color for that slot.
    pub fn from_config(cfg: &ThemeConfig) -> Self {
        Self::from_config_with_base(cfg, Theme::dark())
    }

    pub fn from_config_with_base(cfg: &ThemeConfig, base: Theme) -> Self {
        let tc = detect_truecolor();
        let resolve = |opt: &Option<ColorConfig>, default: Color| -> Color {
            match opt {
                Some(cc) => match cc.to_rgb() {
                    Some((r, g, b)) => {
                        if tc {
                            Color::Rgb(r, g, b)
                        } else {
                            quantize_to_256(r, g, b)
                        }
                    }
                    None => default,
                },
                None => default,
            }
        };

        let sidebar_right = cfg.layout.as_ref()
            .and_then(|l| l.sidebar_position.as_ref())
            .map(|s| s == "right")
            .unwrap_or(base.sidebar_right);
        let search_bottom = cfg.layout.as_ref()
            .and_then(|l| l.search_position.as_ref())
            .map(|s| s == "bottom")
            .unwrap_or(base.search_bottom);

        let border_type = cfg.border.as_ref()
            .and_then(|b| b.border_type.clone())
            .unwrap_or(base.border_type);
        
        let (show_border_top, show_border_bottom, show_border_left, show_border_right) = match &cfg.borders {
            Some(bs) => (bs.show_top, bs.show_bottom, bs.show_left, bs.show_right),
            None => (base.show_border_top, base.show_border_bottom, base.show_border_left, base.show_border_right),
        };

        let active_pointer = cfg.glyphs.as_ref()
            .and_then(|g| g.active_pointer.clone())
            .unwrap_or(base.active_pointer);
        let bullet_marker = cfg.glyphs.as_ref()
            .and_then(|g| g.bullet_marker.clone())
            .unwrap_or(base.bullet_marker);
        let radio_checked = cfg.glyphs.as_ref()
            .and_then(|g| g.radio_checked.clone())
            .unwrap_or(base.radio_checked);
        let radio_unchecked = cfg.glyphs.as_ref()
            .and_then(|g| g.radio_unchecked.clone())
            .unwrap_or(base.radio_unchecked);

        let progress_filled = cfg.progress.as_ref()
            .and_then(|p| p.filled.clone())
            .unwrap_or(base.progress_filled);
        let progress_unfilled = cfg.progress.as_ref()
            .and_then(|p| p.unfilled.clone())
            .unwrap_or(base.progress_unfilled);

        let scrollbar_thumb = cfg.scrollbar.as_ref()
            .and_then(|s| s.thumb.clone())
            .unwrap_or(base.scrollbar_thumb);
        let scrollbar_track = cfg.scrollbar.as_ref()
            .and_then(|s| s.track.clone())
            .unwrap_or(base.scrollbar_track);

        let unread_marker = cfg.status.as_ref()
            .and_then(|s| s.unread_marker.clone())
            .unwrap_or(base.unread_marker);
        let unread_color = resolve(
            &cfg.status.as_ref().and_then(|s| s.unread_color.clone()),
            base.unread_color,
        );
        let reading_marker = cfg.status.as_ref()
            .and_then(|s| s.reading_marker.clone())
            .unwrap_or(base.reading_marker);
        let reading_color = resolve(
            &cfg.status.as_ref().and_then(|s| s.reading_color.clone()),
            base.reading_color,
        );
        let read_marker = cfg.status.as_ref()
            .and_then(|s| s.read_marker.clone())
            .unwrap_or(base.read_marker);
        let read_color = resolve(
            &cfg.status.as_ref().and_then(|s| s.read_color.clone()),
            base.read_color,
        );

        Theme {
            bg: resolve(&cfg.bg.color, base.bg),
            surface: resolve(&cfg.surface, base.surface),
            force_bg: cfg.bg.force,
            fg: resolve(&cfg.fg, base.fg),
            accent_primary: resolve(&cfg.accent_primary, base.accent_primary),
            accent_secondary: resolve(&cfg.accent_secondary, base.accent_secondary),
            dim: resolve(&cfg.dim, base.dim),
            divider: resolve(&cfg.divider, base.divider),
            selected: resolve(&cfg.selected, base.selected),
            focus_bg: resolve(&cfg.focus_bg, base.focus_bg),
            focus_fg: resolve(&cfg.focus_fg, base.focus_fg),
            title: resolve(&cfg.title, base.title),
            meta: resolve(&cfg.meta, base.meta),
            key: resolve(&cfg.key, base.key),
            tag: resolve(&cfg.tag, base.tag),
            udc: resolve(&cfg.udc, base.udc),
            error: resolve(&cfg.error, base.error),
            warning: resolve(&cfg.warning, base.warning),
            code: resolve(&cfg.code, base.code),
            success: resolve(&cfg.success, base.success),
            search_bg: resolve(&cfg.search_bg, base.search_bg),

            sidebar_right,
            search_bottom,
            title_bold: cfg.title_bold.unwrap_or(base.title_bold),
            title_underline: cfg.title_underline.unwrap_or(base.title_underline),
            header_bold: cfg.header_bold.unwrap_or(base.header_bold),
            header_underline: cfg.header_underline.unwrap_or(base.header_underline),

            border_type,
            show_border_top,
            show_border_bottom,
            show_border_left,
            show_border_right,
            active_pointer,
            bullet_marker,
            radio_checked,
            radio_unchecked,
            progress_filled,
            progress_unfilled,
            scrollbar_thumb,
            scrollbar_track,
            unread_marker,
            unread_color,
            reading_marker,
            reading_color,
            read_marker,
            read_color,
        }
    }
}

// ── Global theme (set at startup, updatable at runtime) ──────

static THEME: Lazy<RwLock<Theme>> = Lazy::new(|| RwLock::new(Theme::default()));

/// Initialize or update the global theme. Call at startup and whenever
/// the user changes theme settings in the settings panel.
pub fn init_theme(theme: Theme) {
    *THEME.write().unwrap() = theme;
}

/// Access the global theme. Falls back to `Theme::default()` if not initialized.
fn theme() -> Theme {
    THEME.read().unwrap().clone()
}

// ── Style functions (read from global theme) ──────────────────
// Existing signatures preserved — no call-site changes needed.

pub fn default_style() -> Style {
    let t = theme();
    Style::default().fg(t.fg).bg(t.bg_color())
}

pub fn bg_style() -> Style {
    let t = theme();
    Style::default().bg(t.bg_color()).fg(t.fg)
}

pub fn header_style() -> Style {
    let t = theme();
    let mut style = Style::default().fg(t.accent_primary).bg(t.bg_color());
    if t.header_bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if t.header_underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

pub fn title_style() -> Style {
    let t = theme();
    let mut style = Style::default().fg(t.title);
    if t.title_bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if t.title_underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

pub fn label_style() -> Style {
    let t = theme();
    Style::default().fg(t.accent_primary)
}

pub fn meta_style() -> Style {
    let t = theme();
    Style::default().fg(t.meta)
}

pub fn code_style() -> Style {
    let t = theme();
    Style::default().fg(t.code)
}

pub fn key_style() -> Style {
    let t = theme();
    Style::default().fg(t.key)
}

pub fn focus_style() -> Style {
    let t = theme();
    Style::default()
        .bg(t.focus_bg)
        .fg(t.focus_fg)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    let t = theme();
    Style::default().fg(t.selected)
}

pub fn dim_style() -> Style {
    let t = theme();
    Style::default().fg(t.dim)
}

pub fn divider_style() -> Style {
    let t = theme();
    Style::default().fg(t.divider)
}

pub fn error_style() -> Style {
    let t = theme();
    Style::default().fg(t.error)
}

pub fn success_style() -> Style {
    let t = theme();
    Style::default().fg(t.success)
}

// ── New style functions for previously-unstyled colors ────────

pub fn accent_style() -> Style {
    let t = theme();
    Style::default().fg(t.accent_primary)
}

pub fn tag_style() -> Style {
    let t = theme();
    Style::default().fg(t.tag)
}

pub fn udc_style() -> Style {
    let t = theme();
    Style::default().fg(t.udc)
}

pub fn warning_style() -> Style {
    let t = theme();
    Style::default().fg(t.warning)
}

pub fn brand_style() -> Style {
    let t = theme();
    Style::default()
        .fg(t.accent_secondary)
        .add_modifier(Modifier::BOLD)
}

// ── Color accessors (for inline Color:: replacement) ──────────

pub fn bg() -> Color {
    theme().bg_color()
}
pub fn surface() -> Color {
    theme().surface
}
pub fn fg() -> Color {
    theme().fg
}
pub fn accent_primary() -> Color {
    theme().accent_primary
}
pub fn accent_secondary() -> Color {
    theme().accent_secondary
}
pub fn dim() -> Color {
    theme().dim
}
pub fn divider() -> Color {
    theme().divider
}
pub fn selected() -> Color {
    theme().selected
}
pub fn focus_bg() -> Color {
    theme().focus_bg
}
pub fn focus_fg() -> Color {
    theme().focus_fg
}
pub fn title_fg() -> Color {
    theme().title
}
pub fn meta() -> Color {
    theme().meta
}
pub fn key() -> Color {
    theme().key
}
pub fn tag() -> Color {
    theme().tag
}
pub fn udc() -> Color {
    theme().udc
}
pub fn error() -> Color {
    theme().error
}
pub fn warning() -> Color {
    theme().warning
}
pub fn code() -> Color {
    theme().code
}
pub fn success() -> Color {
    theme().success
}
pub fn search_bg() -> Color {
    theme().search_bg
}

pub fn surface_style() -> Style {
    let t = theme();
    Style::default().fg(t.fg).bg(t.surface)
}

// ── 신규 비주얼 옵션 헬퍼 함수들 ──

pub fn sidebar_right() -> bool {
    theme().sidebar_right
}

pub fn search_bottom() -> bool {
    theme().search_bottom
}

pub fn border_type() -> String {
    theme().border_type.clone()
}

pub fn show_border_top() -> bool {
    theme().show_border_top
}

pub fn show_border_bottom() -> bool {
    theme().show_border_bottom
}

pub fn show_border_left() -> bool {
    theme().show_border_left
}

pub fn show_border_right() -> bool {
    theme().show_border_right
}

pub fn active_pointer() -> String {
    theme().active_pointer.clone()
}

pub fn bullet_marker() -> String {
    theme().bullet_marker.clone()
}

pub fn radio_checked() -> String {
    theme().radio_checked.clone()
}

pub fn radio_unchecked() -> String {
    theme().radio_unchecked.clone()
}

pub fn scrollbar_thumb() -> String {
    theme().scrollbar_thumb.clone()
}

pub fn scrollbar_track() -> String {
    theme().scrollbar_track.clone()
}

pub fn unread_marker() -> String {
    theme().unread_marker.clone()
}

pub fn unread_color() -> Color {
    theme().unread_color
}

pub fn reading_marker() -> String {
    theme().reading_marker.clone()
}

pub fn reading_color() -> Color {
    theme().reading_color
}

pub fn read_marker() -> String {
    theme().read_marker.clone()
}

pub fn read_color() -> Color {
    theme().read_color
}

/// Build a single-line progress bar using `━` (filled) and `─` (unfilled).
/// The filled portion uses `color`, the unfilled portion uses the dim color.
/// `pct` is clamped to 0..=100. Returns a vector of Spans ready to embed in a Line.
pub fn progress_bar_spans(pct: u8, len: usize, color: Color) -> Vec<Span<'static>> {
    let pct = pct.min(100);
    let filled = (pct as usize * len / 100).min(len);
    let t = theme();
    vec![
        Span::styled(t.progress_filled.repeat(filled), Style::default().fg(color)),
        Span::styled(t.progress_unfilled.repeat(len - filled), Style::default().fg(t.dim)),
    ]
}

// ── 테마 플러그인 로딩 로직 ──

pub fn load_theme(cfg: &crate::config::AppConfig) -> Theme {
    let base = match cfg.theme_name.as_str() {
        "light" => Theme::light(),
        "dark" | "" => Theme::dark(),
        custom => {
            if let Some(custom_theme) = load_custom_theme(custom) {
                custom_theme
            } else {
                Theme::dark()
            }
        }
    };
    Theme::from_config_with_base(&cfg.theme, base)
}

fn load_custom_theme(name: &str) -> Option<Theme> {
    // Block path traversal: name must not contain path separators
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        tracing::warn!("custom theme name contains path separators: {name:?}");
        return None;
    }
    let home = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())?;
    let themes_dir = home.join(".libran").join("themes");
    
    if !themes_dir.exists() {
        let _ = fs::create_dir_all(&themes_dir);
        let sample_path = themes_dir.join("sample_custom.toml");
        let sample_content = r##"# 샘플 커스텀 테마 설정 예시
[theme.bg]
color = "#121814"
force = true

[theme]
surface = "#1A231C"
fg = "#D0DCD0"
accent_primary = "#7EA385"
accent_secondary = "#BACD92"
dim = "#607364"
divider = "#2A3A2F"
selected = "#F5F5DC"
focus_bg = "#2A3D2E"
focus_fg = "#E6EFE6"
title = "#E6EFE6"

# 텍스트 스타일 데코레이션 오버라이드
title_bold = true
title_underline = false
header_bold = true
header_underline = false

# 레이아웃 토폴로지 변경 설정
[layout]
sidebar_position = "right"
search_position = "bottom"
"##;
        let _ = fs::write(&sample_path, sample_content);
    }
    
    let path = themes_dir.join(format!("{}.toml", name));
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<ThemeConfig>(&content) {
                return Some(Theme::from_config_with_base(&cfg, Theme::dark()));
            }
        }
    }
    None
}

pub fn get_available_themes() -> Vec<String> {
    let mut themes = vec!["dark".to_string(), "light".to_string()];
    if let Some(home) = directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
        let themes_dir = home.join(".libran").join("themes");
        if themes_dir.exists() {
            if let Ok(entries) = fs::read_dir(themes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem != "sample_custom" && stem != "dark" && stem != "light" {
                                themes.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    themes
}

pub fn create_theme_block(title: &str) -> Block<'static> {
    let t = theme();
    
    // 테두리 방향 구성
    let mut borders = Borders::NONE;
    if t.show_border_top { borders |= Borders::TOP; }
    if t.show_border_bottom { borders |= Borders::BOTTOM; }
    if t.show_border_left { borders |= Borders::LEFT; }
    if t.show_border_right { borders |= Borders::RIGHT; }

    // 테두리 종류 구성
    let b_type = match t.border_type.as_str() {
        "rounded" => BorderType::Rounded,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        _ => BorderType::Plain,
    };

    let block = Block::default()
        .borders(borders)
        .border_style(Style::default().fg(t.accent_primary))
        .style(Style::default().fg(t.fg).bg(t.bg_color()))
        .border_type(b_type);

    if !title.is_empty() {
        block.title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(t.accent_secondary).add_modifier(Modifier::BOLD)
        ))
    } else {
        block
    }
}

// ── Truecolor detection + 256-color fallback ──────────────────

fn detect_truecolor() -> bool {
    env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false)
}

/// Quantize an RGB triple to the nearest 256-color palette index.
/// Uses the 6×6×6 color cube (16–231) for chromatic colors
/// and the 24-step grayscale ramp (232–255) for achromatic colors.
fn quantize_to_256(r: u8, g: u8, b: u8) -> Color {
    // Grayscale ramp: 24 levels from 8 to 238
    if r == g && g == b {
        if r < 8 {
            return Color::Indexed(16); // black
        } else if r > 238 {
            return Color::Indexed(231); // white
        }
        let gray_idx = 232 + ((r as u16 - 8) * 24 / 230) as u8;
        return Color::Indexed(gray_idx);
    }

    // 6×6×6 color cube: levels [0, 95, 135, 175, 215, 255]
    let q = |c: u8| -> u8 {
        if c < 48 {
            0
        } else if c < 115 {
            1
        } else if c < 155 {
            2
        } else if c < 195 {
            3
        } else if c < 235 {
            4
        } else {
            5
        }
    };
    Color::Indexed(16 + 36 * q(r) + 6 * q(g) + q(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_default_has_blue_accent() {
        let t = Theme::default();
        if detect_truecolor() {
            assert_eq!(t.accent_primary, Color::Rgb(107, 155, 210));
        }
    }

    #[test]
    fn theme_force_bg_defaults_to_true() {
        let t = Theme::default();
        assert!(t.force_bg);
    }

    #[test]
    fn theme_bg_color_returns_rgb_when_forced() {
        let t = Theme {
            force_bg: true,
            bg: Color::Rgb(10, 20, 30),
            ..Theme::default()
        };
        assert_eq!(t.bg_color(), Color::Rgb(10, 20, 30));
    }

    #[test]
    fn theme_bg_color_returns_reset_when_not_forced() {
        let t = Theme {
            force_bg: false,
            ..Theme::default()
        };
        assert_eq!(t.bg_color(), Color::Reset);
    }

    #[test]
    fn quantize_black_is_indexed_16() {
        let c = quantize_to_256(0, 0, 0);
        assert_eq!(c, Color::Indexed(16));
    }

    #[test]
    fn quantize_white_is_indexed_231() {
        let c = quantize_to_256(255, 255, 255);
        assert_eq!(c, Color::Indexed(231));
    }

    #[test]
    fn quantize_gray_is_grayscale_ramp() {
        let c = quantize_to_256(128, 128, 128);
        assert!(matches!(c, Color::Indexed(idx) if idx >= 232 && idx <= 255));
    }

    #[test]
    fn quantize_red_is_color_cube() {
        let c = quantize_to_256(205, 0, 0);
        // 205 → q=4, 0 → q=0, 0 → q=0 → 16 + 36*4 = 160
        assert_eq!(c, Color::Indexed(160));
    }
}
