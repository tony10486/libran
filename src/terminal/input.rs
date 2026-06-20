use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;

use crate::app::AppAction;
use crate::terminal::drag_drop;

pub async fn poll_event(
    stream: &mut EventStream,
) -> Option<AppAction> {
    while let Some(Ok(event)) = stream.next().await {
        if let Some(action) = convert_event(&event) {
            return Some(action);
        }
    }
    None
}

fn convert_event(event: &Event) -> Option<AppAction> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Release {
                return None;
            }
            Some(AppAction::KeyPressed(*key))
        }
        Event::Paste(text) => {
            drag_drop::parse_dragged_path(text).map(AppAction::DragDetected)
        }
        _ => None,
    }
}

pub fn key_code_char(key: &KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(c) => Some(c),
        _ => None,
    }
}
