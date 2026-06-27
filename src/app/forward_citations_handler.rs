use anyhow::Result;
use rusqlite::Connection;

use crate::api::openalex_forward::{self, ForwardCitation};
use crate::app::action::AppAction;
use crate::app::AppState;
use crate::db::documents::{self, Document};

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
                    .send(AppAction::ForwardCitationsFailed { doc_id, reason: e })
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
            Ok((citations, cited_by_count)) => {
                // Lock is taken in a non-async block so the MutexGuard drops before the .await below.
                match db.lock() {
                    Ok(conn) => {
                        if let Err(e) =
                            persist_forward_citations(&conn, doc_id, &citations)
                        {
                            eprintln!("forward citations persist failed: {e}");
                        }
                    }
                    Err(e) => eprintln!("DB lock failed for forward citations persist: {e}"),
                }
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

/// Persist forward citations: for each citing work, reuse an existing document
/// when its DOI is already in the DB (dedup), otherwise insert a new document
/// with `source = "openalex_forward"`. Then add a `citation_relations` edge
/// (citing -> cited) per citation. Edge dedup is enforced at the DB layer.
pub fn persist_forward_citations(
    conn: &Connection,
    cited_doc_id: i64,
    citations: &[ForwardCitation],
) -> Result<()> {
    for cite in citations {
        let citing_doc_id = match cite.doi.as_deref() {
            Some(doi) if !doi.is_empty() => {
                documents::find_by_doi(conn, doi)?.and_then(|d| d.id)
            }
            _ => None,
        };

        let citing_doc_id = match citing_doc_id {
            Some(id) => id,
            None => {
                let authors = if cite.authors.is_empty() {
                    None
                } else {
                    Some(cite.authors.join("; "))
                };
                let doc = Document {
                    id: None,
                    title: cite.title.clone(),
                    authors,
                    doi: cite.doi.clone(),
                    pub_year: cite.year,
                    citation_key: None,
                    source: Some("openalex_forward".to_string()),
                    ..Default::default()
                };
                documents::insert(conn, &doc)?
            }
        };

        documents::add_citation(conn, citing_doc_id, cited_doc_id)?;
    }
    Ok(())
}
