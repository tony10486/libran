use crate::app::AppState;
use crate::db::saved_searches;

/// Enter save-search naming mode.
pub fn handle_save_current_search(state: &mut AppState) {
    if state.search_input.is_empty() {
        state.set_status("저장할 검색어가 없습니다");
        return;
    }
    state.save_search_mode = true;
    state.save_search_input.clear();
    state.set_status("저장할 검색 이름 입력 (Esc 취소)");
}

/// Save the current search with the given name.
pub fn handle_save_current_search_named(state: &mut AppState, name: String) {
    let query = state.search_input.clone();
    let db = state.db.clone();

    if let Ok(conn) = db.lock() {
        if let Err(e) = saved_searches::insert(&conn, &name, Some(&query), Some("{}")) {
            state.set_status(&format!("저장 실패: {}", e));
        } else {
            state.set_status(&format!("검색 저장됨: {}", name));
        }
        drop(conn);
    }
    state.save_search_mode = false;
    state.save_search_input.clear();
    state.reload_saved_searches();
}

/// Apply a saved search.
pub fn handle_select_saved_search(state: &mut AppState, search_id: i64) {
    let db = state.db.clone();
    let apply_result: Option<(String, String)> = if let Ok(conn) = db.lock() {
        saved_searches::get_by_id(&conn, search_id)
            .ok()
            .flatten()
            .and_then(|s| s.fts_query.map(|q| (s.name, q)))
    } else {
        None
    };

    if let Some((name, query)) = apply_result {
        state.search_input = query;
        state.set_status(&format!("검색 적용: {}", name));
        state.reload_documents();
    } else {
        state.set_status("저장된 검색을 찾을 수 없음");
    }
}

/// Delete a saved search.
pub fn handle_delete_saved_search(state: &mut AppState, search_id: i64) {
    let db = state.db.clone();
    if let Ok(conn) = db.lock() {
        if saved_searches::delete(&conn, search_id).is_ok() {
            state.set_status("저장된 검색 삭제됨");
        } else {
            state.set_status("삭제 실패");
        }
        drop(conn);
    }
    state.reload_saved_searches();
}
