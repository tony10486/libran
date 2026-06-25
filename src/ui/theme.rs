use ratatui::style::{Color, Modifier, Style};
use std::env;
use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::config::{ColorConfig, ThemeConfig};

// ── Theme struct ──────────────────────────────────────────────

/// Central theme with 18 semantic color slots.
/// All colors are stored as `Color::Rgb` on truecolor terminals,
/// or `Color::Indexed` (quantized) on 256-color terminals.
/// Named colors (`Color::Cyan`, `Color::Black`, etc.) are never used
/// because terminals can remap them, defeating forced backgrounds.
#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    /// Slate RGB(148,163,184) — replaces Cyan. Used for headers, labels, structure.
    pub accent_primary: Color,
    /// White — used for brand/active emphasis with BOLD.
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
}

impl Theme {
    pub fn default() -> Self {
        let tc = detect_truecolor();
        let rgb = |r: u8, g: u8, b: u8| -> Color {
            if tc {
                Color::Rgb(r, g, b)
            } else {
                quantize_to_256(r, g, b)
            }
        };
        Theme {
            bg: rgb(0, 0, 0),
            fg: rgb(128, 128, 128),
            accent_primary: rgb(148, 163, 184),
            accent_secondary: rgb(255, 255, 255),
            dim: rgb(85, 85, 85),
            divider: rgb(85, 85, 85),
            selected: rgb(205, 205, 0),
            focus_bg: rgb(50, 50, 50),
            focus_fg: rgb(255, 255, 255),
            title: rgb(255, 255, 255),
            meta: rgb(128, 128, 128),
            key: rgb(0, 205, 0),
            tag: rgb(205, 0, 205),
            udc: rgb(0, 0, 205),
            error: rgb(205, 0, 0),
            warning: rgb(205, 133, 0),
            code: rgb(205, 205, 0),
            success: rgb(0, 205, 0),
            search_bg: rgb(48, 48, 48),
            force_bg: true,
        }
    }

    /// Returns the configured bg color if `force_bg` is true,
    /// otherwise `Color::Reset` (terminal default).
    pub fn bg_color(&self) -> Color {
        if self.force_bg {
            self.bg
        } else {
            Color::Reset
        }
    }

    /// Build a Theme from ThemeConfig, filling unset slots with defaults.
    /// Invalid hex strings fall back to the default color for that slot.
    pub fn from_config(cfg: &ThemeConfig) -> Self {
        let default = Theme::default();
        let tc = detect_truecolor();
        let resolve = |opt: &Option<ColorConfig>, default: Color| -> Color {
            match opt {
                Some(cc) => match cc.to_rgb() {
                    Some((r, g, b)) => {
                        if tc { Color::Rgb(r, g, b) } else { quantize_to_256(r, g, b) }
                    }
                    None => default,
                },
                None => default,
            }
        };
        Theme {
            bg: resolve(&cfg.bg.color, default.bg),
            force_bg: cfg.bg.force,
            fg: resolve(&cfg.fg, default.fg),
            accent_primary: resolve(&cfg.accent_primary, default.accent_primary),
            accent_secondary: resolve(&cfg.accent_secondary, default.accent_secondary),
            dim: resolve(&cfg.dim, default.dim),
            divider: resolve(&cfg.divider, default.divider),
            selected: resolve(&cfg.selected, default.selected),
            focus_bg: resolve(&cfg.focus_bg, default.focus_bg),
            focus_fg: resolve(&cfg.focus_fg, default.focus_fg),
            title: resolve(&cfg.title, default.title),
            meta: resolve(&cfg.meta, default.meta),
            key: resolve(&cfg.key, default.key),
            tag: resolve(&cfg.tag, default.tag),
            udc: resolve(&cfg.udc, default.udc),
            error: resolve(&cfg.error, default.error),
            warning: resolve(&cfg.warning, default.warning),
            code: resolve(&cfg.code, default.code),
            success: resolve(&cfg.success, default.success),
            search_bg: resolve(&cfg.search_bg, default.search_bg),
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
    *THEME.read().unwrap()
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
    Style::default()
        .fg(t.accent_primary)
        .add_modifier(Modifier::BOLD)
        .bg(t.bg_color())
}

pub fn title_style() -> Style {
    let t = theme();
    Style::default().fg(t.title).add_modifier(Modifier::BOLD)
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
    fn theme_default_has_slate_accent() {
        let t = Theme::default();
        // On truecolor terminals, accent_primary should be Slate RGB
        if detect_truecolor() {
            assert_eq!(t.accent_primary, Color::Rgb(148, 163, 184));
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
