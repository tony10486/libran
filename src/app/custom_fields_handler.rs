use crossterm::event::{KeyCode, KeyEvent};

use crate::db::documents::Document;

use super::AppState;
use super::action::AppAction;

pub(crate) fn handle_select_udc(state: &mut AppState, notation: Option<String>) {
    state.active_udc_notation = notation.clone();
    state.active_project_id = None;
    state.active_author = None;
    state.active_series_id = None;
    if let Some(ref notation) = notation {
        state.expanded_nodes.insert(notation.clone());
        if let Ok(conn) = state.db.lock() {
            let node_id = crate::classification::scheme::get_node_id(&conn, "udc", notation)
                .ok()
                .flatten();
            let all = crate::db::documents::list_all(&conn).unwrap_or_default();
            let filtered: Vec<Document> = match node_id {
                Some(node_id) => all
                    .into_iter()
                    .filter(|d| {
                        conn.query_row(
                            "SELECT COUNT(*) FROM document_classifications WHERE document_id = ?1 AND node_id = ?2",
                            rusqlite::params![d.id.unwrap_or(0), node_id],
                            |row| row.get::<_, i64>(0),
                        )
                        .unwrap_or(0)
                            > 0
                    })
                    .collect(),
                None => Vec::new(),
            };
            let count = filtered.len();
            state.documents = filtered;
            state.document_count = count;
            state.list_cursor = 0;
        }
        state.set_status(&format!("UDC {} 분류 문헌", notation));
    } else {
        state.reload_documents();
    }
    state.dirty = true;
}

pub(crate) fn handle_add_custom_field(
    state: &mut AppState,
    doc_id: i64,
    key: String,
    value: String,
) {
    if key.trim().is_empty() {
        state.set_status("필드 키를 입력하세요");
        return;
    }
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::custom_fields::add_field(&conn, doc_id, key.trim(), value.trim())
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            state.load_detail();
            state.set_status("커스텀 필드 추가됨");
        }
        Err(e) => state.set_status(&format!("필드 추가 실패: {}", e)),
    }
    state.dirty = true;
}

pub(crate) fn handle_delete_custom_field(state: &mut AppState, doc_id: i64, field_id: i64) {
    let result = {
        if let Ok(conn) = state.db.lock() {
            crate::db::custom_fields::delete_field(&conn, doc_id, field_id)
        } else {
            Err(anyhow::anyhow!("DB 락 획득 실패"))
        }
    };
    match result {
        Ok(_) => {
            state.load_detail();
            state.set_status("커스텀 필드 삭제됨");
        }
        Err(e) => state.set_status(&format!("필드 삭제 실패: {}", e)),
    }
    state.dirty = true;
}

pub(crate) fn handle_custom_field_key(state: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            state.custom_field_mode = false;
            state.custom_field_key.clear();
            state.custom_field_value.clear();
            state.custom_field_editing_key = false;
            state.dirty = true;
        }
        KeyCode::Tab => {
            state.custom_field_editing_key = !state.custom_field_editing_key;
            state.dirty = true;
        }
        KeyCode::Enter => {
            let doc_id = state.detail_doc.as_ref().and_then(|d| d.id).unwrap_or(0);
            let key = state.custom_field_key.clone();
            let value = state.custom_field_value.clone();
            state.custom_field_mode = false;
            state.custom_field_key.clear();
            state.custom_field_value.clear();
            state.custom_field_editing_key = false;
            let _ = state
                .action_tx
                .try_send(AppAction::AddCustomField { doc_id, key, value });
        }
        KeyCode::Backspace => {
            if state.custom_field_editing_key {
                state.custom_field_key.pop();
            } else {
                state.custom_field_value.pop();
            }
            state.dirty = true;
        }
        KeyCode::Char(c) => {
            if state.custom_field_editing_key {
                if c.is_ascii_graphic() || c == ' ' {
                    state.custom_field_key.push(c);
                }
            } else {
                state.custom_field_value.push(c);
            }
            state.dirty = true;
        }
        _ => {}
    }
    false
}
