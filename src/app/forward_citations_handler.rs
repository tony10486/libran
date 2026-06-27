use crate::api::openalex_forward::{self, ForwardCitation};
use crate::app::AppState;
use crate::app::action::AppAction;
use crate::db::documents;
use anyhow::Result;
use rusqlite::Connection;

/// Persist forward citations as documents + citation_relations edges.
///
/// `cited_doc_id` is the document whose forward citations were fetched.
/// Each `ForwardCitation` is a paper that cites it. The citing paper is
/// upserted as a document (deduped by DOI) and a `citation_relations` edge
/// (citing → cited) is inserted.
pub fn persist_forward_citations(
    conn: &Connection,
    cited_doc_id: i64,
    citations: &[ForwardCitation],
) -> Result<()> {
    for citation in citations {
        let citing_id = match citation.doi.as_deref() {
            Some(doi) if !doi.is_empty() => match documents::find_by_doi(conn, doi)? {
                Some(existing) => existing.id.unwrap(),
                None => insert_forward_citation_doc(conn, citation)?,
            },
            _ => insert_forward_citation_doc(conn, citation)?,
        };
        documents::add_citation(conn, citing_id, cited_doc_id)?;
    }
    Ok(())
}

fn insert_forward_citation_doc(conn: &Connection, citation: &ForwardCitation) -> Result<i64> {
    let doc = documents::Document {
        title: citation.title.clone(),
        authors: if citation.authors.is_empty() {
            None
        } else {
            Some(citation.authors.join("; "))
        },
        pub_year: citation.year,
        doi: citation.doi.clone(),
        source: Some("openalex_forward".to_string()),
        ..Default::default()
    };
    documents::insert(conn, &doc)
}

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
                if let Ok(conn) = db.lock() {
                    let _ = persist_forward_citations(&conn, doc_id, &citations);
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
