use crate::app::action::AppAction;
use crate::app::AppState;
use crate::db::documents;
use crate::pdf::bookmarks;
use crate::storage::library;

use std::path::Path;

/// Start bookmark extraction (async).
pub fn handle_extract_bookmarks(state: &mut AppState, doc_id: i64) {
    state.start_processing("북마크 추출 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<(String, i64)>, String> {
            let conn = db.lock().map_err(|e| e.to_string())?;
            let doc = documents::get_by_id(&conn, doc_id)
                .map_err(|e| e.to_string())?
                .ok_or("문헌을 찾을 수 없음")?;
            let file_path = doc.file_path.ok_or("파일 경로 없음")?;
            let path = Path::new(&file_path);
            if !library::check_file_exists(path) {
                return Err("파일을 찾을 수 없음".to_string());
            }
            bookmarks::extract_bookmarks(path).map_err(|e| e.to_string())
        })
        .await;

        let action = match result {
            Ok(Ok(bms)) => AppAction::BookmarksExtracted { doc_id, bookmarks: bms },
            Ok(Err(m)) => AppAction::BookmarkExtractionFailed { doc_id, reason: m },
            Err(e) => AppAction::BookmarkExtractionFailed {
                doc_id,
                reason: format!("태스크 실패: {}", e),
            },
        };
        let _ = tx.send(action).await;
    });
}

/// Store extracted bookmarks in state.
pub fn handle_bookmarks_extracted(
    state: &mut AppState,
    _doc_id: i64,
    bms: Vec<(String, i64)>,
) {
    state.finish_processing(&format!("북마크 {}개 추출됨", bms.len()));
    state.current_bookmarks = bms;
}

/// Show extraction failure.
pub fn handle_bookmark_extraction_failed(state: &mut AppState, _doc_id: i64, reason: String) {
    state.finish_processing(&format!("북마크 추출 실패: {}", reason));
    state.current_bookmarks.clear();
}
