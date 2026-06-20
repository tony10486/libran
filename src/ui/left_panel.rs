use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Libran ");

    let items: Vec<ListItem> = if state.is_processing {
        vec![ListItem::new(format!("처리 중: {}", state.status_text))]
    } else {
        vec![
            ListItem::new("프로젝트"),
            ListItem::new(""),
            ListItem::new("분류 (UDC)"),
            ListItem::new("  0 총류"),
            ListItem::new("  1 철학"),
            ListItem::new("  2 종교"),
            ListItem::new("  3 사회과학"),
            ListItem::new("  5 자연과학"),
            ListItem::new("  6 응용과학"),
            ListItem::new("  7 예술"),
            ListItem::new("  8 언어"),
            ListItem::new("  9 역사"),
            ListItem::new(""),
            ListItem::new("[Tab] 패널 이동"),
        ]
    };

    let list = List::default().items(items).block(block);
    frame.render_widget(list, area);
}
