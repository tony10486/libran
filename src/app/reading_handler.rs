use crate::app::AppState;
use crate::db::documents;

/// Toggle reading status: unread → reading → read → unread.
/// Synchronous DB update (quick operation).
pub fn handle_toggle_reading_status(state: &mut AppState, doc_id: i64) {
    let db = state.db.clone();
    let result: Result<String, String> = (|| {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let doc = documents::get_by_id(&conn, doc_id)
            .map_err(|e| e.to_string())?
            .ok_or("문헌을 찾을 수 없음".to_string())?;
        let current = doc.reading_status.as_deref().unwrap_or("unread");
        let next = match current {
            "unread" => "reading",
            "reading" => "read",
            _ => "unread",
        };
        documents::update_reading_status(&conn, doc_id, next).map_err(|e| e.to_string())?;
        let label = match next {
            "reading" => "읽는 중",
            "read" => "읽음",
            _ => "안 읽음",
        };
        Ok(label.to_string())
    })();

    match result {
        Ok(label) => {
            state.set_status(&format!("읽기 상태: {}", label));
            state.reload_documents();
            state.load_detail();
        }
        Err(e) => {
            state.set_status(&format!("읽기 상태 변경 실패: {}", e));
        }
    }
}
