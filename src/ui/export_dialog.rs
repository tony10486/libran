use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
use crate::ui::theme;
use crate::export::ExportFormat;
use crate::export::export_dialog_state::DialogSection;

const VISIBLE_ITEMS: usize = 5;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);

    let dialog = &state.export_dialog_state;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(popup);

    render_title(frame, chunks[0]);
    render_format_section(frame, chunks[1], dialog);
    render_style_section(frame, chunks[2], dialog);
    render_language_display_section(frame, chunks[3], dialog);
    render_preview(frame, chunks[4], dialog);
    render_footer(frame, chunks[5]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![Span::styled(
        "  내보내기 / Export",
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]);
    frame.render_widget(Paragraph::new(vec![Line::from(""), line]), area);
}

fn render_format_section(
    frame: &mut Frame,
    area: Rect,
    dialog: &crate::export::export_dialog_state::ExportDialogState,
) {
    let focused = dialog.focused_section == DialogSection::Format;
    let items = build_section_items(
        ExportFormat::all(),
        dialog.selected_format,
        dialog.format_cursor,
        focused,
        |f| f.is_implemented(),
        |f| f.format_name(),
    );

    let lines = std::iter::once(header_line("형식 (Format)", focused))
        .chain(std::iter::once(Line::from("")))
        .chain(items.into_iter())
        .collect::<Vec<_>>();

    let style = if focused {
        Style::default().fg(theme::accent_primary()).bg(theme::bg())
    } else {
        Style::default().fg(theme::fg()).bg(theme::bg())
    };

    frame.render_widget(Paragraph::new(lines).style(style), area);
}

fn render_style_section(
    frame: &mut Frame,
    area: Rect,
    dialog: &crate::export::export_dialog_state::ExportDialogState,
) {
    let focused = dialog.focused_section == DialogSection::Style;
    let items = build_section_items(
        CitationStyle::all(),
        dialog.selected_style,
        dialog.style_cursor,
        focused,
        |s| s.is_implemented(),
        |s| s.display_name(),
    );

    let lines = std::iter::once(header_line("인용 스타일 (Citation Style)", focused))
        .chain(std::iter::once(Line::from("")))
        .chain(items.into_iter())
        .collect::<Vec<_>>();

    let style = if focused {
        Style::default().fg(theme::accent_primary()).bg(theme::bg())
    } else {
        Style::default().fg(theme::fg()).bg(theme::bg())
    };

    frame.render_widget(Paragraph::new(lines).style(style), area);
}

fn render_language_display_section(
    frame: &mut Frame,
    area: Rect,
    dialog: &crate::export::export_dialog_state::ExportDialogState,
) {
    let lang_focused = dialog.focused_section == DialogSection::Language;
    let display_focused = dialog.focused_section == DialogSection::DisplayMode;
    let display_active = dialog.is_display_mode_active();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let lang_items = build_section_items(
        CitationLanguage::all(),
        dialog.selected_language,
        dialog.language_cursor,
        lang_focused,
        |_| true,
        |l| l.display_name(),
    );

    let lang_lines = std::iter::once(header_line("언어 (Language)", lang_focused))
        .chain(std::iter::once(Line::from("")))
        .chain(lang_items.into_iter())
        .collect::<Vec<_>>();

    let lang_style = if lang_focused {
        Style::default().fg(theme::accent_primary()).bg(theme::bg())
    } else {
        Style::default().fg(theme::fg()).bg(theme::bg())
    };

    frame.render_widget(Paragraph::new(lang_lines).style(lang_style), cols[0]);

    if display_active {
        let modes = [DisplayMode::InText, DisplayMode::Footnotes, DisplayMode::Endnotes];
        let mode_items = build_section_items(
            &modes,
            dialog.display_mode,
            dialog.display_mode_cursor,
            display_focused,
            |_| true,
            |m| m.display_name(),
        );

        let display_lines = std::iter::once(header_line("표시 (Display)", display_focused))
            .chain(std::iter::once(Line::from("")))
            .chain(mode_items.into_iter())
            .collect::<Vec<_>>();

        let display_style = if display_focused {
            Style::default().fg(theme::accent_primary()).bg(theme::bg())
        } else {
            Style::default().fg(theme::fg()).bg(theme::bg())
        };

        frame.render_widget(Paragraph::new(display_lines).style(display_style), cols[1]);
    } else {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  ▸ 표시 (Display)",
                Style::default().fg(theme::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  (비활성 — 노트 기반 스타일만)",
                Style::default().fg(theme::dim()),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme::bg())),
            cols[1],
        );
    }
}

fn render_preview(
    frame: &mut Frame,
    area: Rect,
    dialog: &crate::export::export_dialog_state::ExportDialogState,
) {
    let lines = vec![
        Line::from(Span::styled(
            "  ── 미리보기 (Preview) ──",
            Style::default().fg(theme::accent_primary()).add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(Span::styled(
            dialog.preview_text.clone(),
            Style::default().fg(theme::selected()),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(theme::bg()))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("  Enter", Style::default().fg(theme::accent_primary())),
        Span::styled(" 복사  ", Style::default().fg(theme::dim())),
        Span::styled("e", Style::default().fg(theme::accent_primary())),
        Span::styled(" 내보내기  ", Style::default().fg(theme::dim())),
        Span::styled("Tab", Style::default().fg(theme::accent_primary())),
        Span::styled(" 다음  ", Style::default().fg(theme::dim())),
        Span::styled("Esc", Style::default().fg(theme::accent_primary())),
        Span::styled(" 취소", Style::default().fg(theme::dim())),
    ]);
    frame.render_widget(Paragraph::new(vec![Line::from(""), line]), area);
}

fn header_line(name: &str, focused: bool) -> Line<'static> {
    let style = if focused {
        Style::default()
            .fg(theme::accent_primary())
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
    } else {
        Style::default().fg(theme::accent_primary()).add_modifier(Modifier::UNDERLINED)
    };
    Line::from(Span::styled(format!("  ▸ {}", name), style))
}

fn build_section_items<T: Copy + PartialEq>(
    all: &[T],
    selected: T,
    cursor: usize,
    focused: bool,
    is_implemented: impl Fn(&T) -> bool,
    display: impl Fn(&T) -> &str,
) -> Vec<Line<'static>> {
    let total = all.len();
    let start = if total <= VISIBLE_ITEMS {
        0
    } else if cursor < VISIBLE_ITEMS / 2 {
        0
    } else if cursor > total - VISIBLE_ITEMS / 2 - 1 {
        total - VISIBLE_ITEMS
    } else {
        cursor - VISIBLE_ITEMS / 2
    };

    let end = (start + VISIBLE_ITEMS).min(total);

    (start..end)
        .map(|i| {
            let item = all[i];
            let is_current = item == selected;
            let implemented = is_implemented(&item);
            let prefix = if focused && is_current {
                "  ► "
            } else {
                "    "
            };
            let name = display(&item).to_string();
            let marker = if implemented { "" } else { " (미구현)" };

            let style = if !implemented {
                Style::default().fg(theme::dim())
            } else if focused && is_current {
                Style::default().fg(theme::accent_primary()).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(theme::selected())
            } else {
                Style::default().fg(theme::fg())
            };

            Line::from(Span::styled(format!("{}{}{}", prefix, name, marker), style))
        })
        .collect()
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
