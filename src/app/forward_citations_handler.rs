use crate::app::action::AppAction;
use crate::app::AppState;
use crate::api::openalex_forward;
use crate::db::documents;

/// Fetch forward citations from OpenAlex (async).
pub fn handle_fetch_forward_citations(state: &mut AppState, doc_id: i64) {
    state.start_processing("전방 인용 조회 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        // Extract DOI in a match so MutexGuard/PoisonError is dropped before any .await
        let doi_result: Result<Option<String>, String> = match db.lock() {
            Ok(conn) => Ok(documents::get_by_id(&conn, doc_id)
                .ok()
                .flatten()
                .and_then(|d| d.doi)),
            Err(e) => Err(format!("DB 잠금 실패: {}", e)),
        };

        let doi = match doi_result {
            Err(e) => {
                let _ = tx
                    .send(AppAction::ForwardCitationsFailed {
                        doc_id,
                        reason: e,
                    })
                    .await;
                return;
            }
            Ok(Some(d)) if !d.is_empty() => d,
            _ => {
                let _ = tx
                    .send(AppAction::ForwardCitationsFailed {
                        doc_id,
                        reason: "DOI 없음".to_string(),
                    })
                    .await;
                return;
            }
        };

        match openalex_forward::fetch_forward_citations(&doi).await {
            Ok((_citations, cited_by_count)) => {
                let _ = tx
                    .send(AppAction::ForwardCitationsFetched {
                        doc_id,
                        count: cited_by_count,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AppAction::ForwardCitationsFailed {
                        doc_id,
                        reason: e.to_string(),
                    })
                    .await;
            }
        }
    });
}

/// Show forward citations result.
pub fn handle_forward_citations_fetched(state: &mut AppState, _doc_id: i64, count: i64) {
    state.finish_processing(&format!("전방 인용: {}건", count));
}

/// Show forward citations failure.
pub fn handle_forward_citations_failed(state: &mut AppState, _doc_id: i64, reason: String) {
    state.finish_processing(&format!("전방 인용 조회 실패: {}", reason));
}
