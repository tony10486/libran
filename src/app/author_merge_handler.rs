use crate::app::AppState;
use crate::app::action::AppAction;
use crate::db::author_aliases;

/// Enter author merge mode (phase 1: source author).
pub fn handle_start_author_merge(state: &mut AppState) {
    state.author_merge_mode = true;
    state.author_merge_phase = 1;
    state.author_merge_source.clear();
    state.author_merge_input.clear();
    state.set_status("병합할 저자명 입력 (Esc 취소)");
}

/// Store source author, move to phase 2 (canonical author).
pub fn handle_author_merge_source_entered(state: &mut AppState, source: String) {
    state.author_merge_source = source;
    state.author_merge_phase = 2;
    state.author_merge_input.clear();
    state.set_status("병합 대상(정식) 저자명 입력 (Esc 취소)");
}

/// Perform the merge (async).
pub fn handle_author_merge_canonical_entered(
    state: &mut AppState,
    source: String,
    canonical: String,
) {
    state.author_merge_mode = false;
    state.author_merge_phase = 0;
    state.author_merge_input.clear();
    state.start_processing("저자 병합 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();
    let source_display = source.clone();
    let canonical_display = canonical.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> Result<usize, String> {
            let conn = db.lock().map_err(|e| e.to_string())?;
            author_aliases::insert(&conn, &source, &canonical, None).map_err(|e| e.to_string())?;
            let count = author_aliases::merge_author_in_documents(&conn, &source, &canonical)
                .map_err(|e| e.to_string())?;
            Ok(count)
        })
        .await;

        let action = match result {
            Ok(Ok(count)) => AppAction::AuthorMergeResult {
                success: true,
                message: format!(
                    "{}건의 저자 병합 완료: {} → {}",
                    count, source_display, canonical_display
                ),
            },
            Ok(Err(m)) => AppAction::AuthorMergeResult {
                success: false,
                message: format!("병합 실패: {}", m),
            },
            Err(e) => AppAction::AuthorMergeResult {
                success: false,
                message: format!("태스크 실패: {}", e),
            },
        };
        let _ = tx.send(action).await;
    });
}

/// Show merge result.
pub fn handle_author_merge_result(state: &mut AppState, _success: bool, message: String) {
    state.finish_processing(&message);
    state.reload_documents();
}
