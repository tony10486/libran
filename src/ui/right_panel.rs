use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, _state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 문헌 리스트 ");

    let items: Vec<ListItem> = vec![
        ListItem::new("[Space] 선택  [o] 온라인 조회"),
        ListItem::new("[e] 편집  [d] 삭제  [x] 내보내기"),
    ];

    let list = List::default().items(items).block(block);
    frame.render_widget(list, area);
}
