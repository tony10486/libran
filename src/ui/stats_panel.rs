use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::AppState;
use crate::db::stats::LibraryStats;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, stats: &LibraryStats) {
    frame.render_widget(Clear, area);

    let popup = centered_rect(90, 85, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::accent_primary()))
        .title(Span::styled(
            " 📊 라이브러리 통계 (i로 닫기) ",
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::bg()));

    frame.render_widget(block, popup);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(popup);

    render_summary(frame, inner[0], stats);
    render_reading_status(frame, inner[1], stats);
    render_charts(frame, inner[2], stats);
}

fn render_summary(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let lines = vec![
        Line::from(vec![
            Span::styled("  총 문헌      ", theme::label_style()),
            Span::styled(
                stats.total_documents.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
            Span::raw("   "),
            Span::styled("파일 있음 ", theme::label_style()),
            Span::styled(
                stats.documents_with_files.to_string(),
                Style::default().fg(theme::key()),
            ),
            Span::raw("   "),
            Span::styled("DOI ", theme::label_style()),
            Span::styled(
                stats.documents_with_doi.to_string(),
                Style::default().fg(theme::key()),
            ),
            Span::raw("   "),
            Span::styled("arXiv ", theme::label_style()),
            Span::styled(
                stats.documents_with_arxiv.to_string(),
                Style::default().fg(theme::key()),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  태그 ", theme::label_style()),
            Span::styled(
                stats.total_tags.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
            Span::raw("   "),
            Span::styled("프로젝트 ", theme::label_style()),
            Span::styled(
                stats.total_projects.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
            Span::raw("   "),
            Span::styled("시리즈 ", theme::label_style()),
            Span::styled(
                stats.total_series.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
            Span::raw("   "),
            Span::styled("저자 ", theme::label_style()),
            Span::styled(
                stats.total_authors.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
            Span::raw("   "),
            Span::styled("인용관계 ", theme::label_style()),
            Span::styled(
                stats.total_citation_relations.to_string(),
                Style::default().fg(theme::title_fg()),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  별점 문헌 ", theme::label_style()),
            Span::styled(
                format!(
                    "{}건 / 평균 {:.1}점",
                    stats.rated_documents, stats.average_rating
                ),
                Style::default().fg(theme::selected()),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_reading_status(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let total = stats.total_documents.max(1) as f64;
    let unread_pct = (stats.reading_unread as f64 / total * 100.0) as u32;
    let reading_pct = (stats.reading_reading as f64 / total * 100.0) as u32;
    let read_pct = (stats.reading_read as f64 / total * 100.0) as u32;

    let lines = vec![
        Line::from(vec![Span::styled(
            "  ─── 읽기 상태 ───",
            theme::dim_style(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  안 읽음 ", theme::label_style()),
            Span::styled(
                format!("{} ({}%)", stats.reading_unread, unread_pct),
                Style::default().fg(theme::error()),
            ),
            Span::raw("   "),
            Span::styled("읽는 중 ", theme::label_style()),
            Span::styled(
                format!("{} ({}%)", stats.reading_reading, reading_pct),
                Style::default().fg(theme::warning()),
            ),
            Span::raw("   "),
            Span::styled("읽음 ", theme::label_style()),
            Span::styled(
                format!("{} ({}%)", stats.reading_read, read_pct),
                Style::default().fg(theme::success()),
            ),
        ]),
    ];

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_charts(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    render_year_distribution(frame, chunks[0], stats);
    render_top_authors(frame, chunks[1], stats);
    render_top_journals(frame, chunks[2], stats);
}

fn render_year_distribution(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "  ─── 연도별 분포 ───",
            theme::dim_style(),
        )]),
        Line::from(""),
    ];

    if stats.year_distribution.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (데이터 없음)",
            theme::dim_style(),
        )]));
    } else {
        let max_count = stats
            .year_distribution
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(1)
            .max(1);

        for (year, count) in stats.year_distribution.iter().rev().take(20) {
            let bar_len = (*count as f64 / max_count as f64 * 20.0) as usize;
            let bar = "█".repeat(bar_len);
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", year), theme::label_style()),
                Span::styled(bar, Style::default().fg(theme::accent_primary())),
                Span::raw(format!(" {}", count)),
            ]));
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_top_authors(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "  ─── 상위 저자 ───",
            theme::dim_style(),
        )]),
        Line::from(""),
    ];

    if stats.top_authors.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (데이터 없음)",
            theme::dim_style(),
        )]));
    } else {
        for (name, count) in stats.top_authors.iter().take(10) {
            let display_name = if name.len() > 25 {
                format!("{}...", &name[..25])
            } else {
                name.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:3} ", count),
                    Style::default().fg(theme::selected()),
                ),
                Span::styled(display_name, Style::default().fg(theme::fg())),
            ]));
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_top_journals(frame: &mut Frame, area: Rect, stats: &LibraryStats) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "  ─── 상위 저널 ───",
            theme::dim_style(),
        )]),
        Line::from(""),
    ];

    if stats.top_journals.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (데이터 없음)",
            theme::dim_style(),
        )]));
    } else {
        for (name, count) in stats.top_journals.iter().take(10) {
            let display_name = if name.len() > 25 {
                format!("{}...", &name[..25])
            } else {
                name.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:3} ", count),
                    Style::default().fg(theme::selected()),
                ),
                Span::styled(display_name, Style::default().fg(theme::fg())),
            ]));
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
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
