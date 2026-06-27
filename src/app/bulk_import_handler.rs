use crate::api;
use crate::app::AppState;
use crate::app::action::AppAction;
use crate::app::api_metadata;
use crate::db::documents;
use crate::pdf;

/// Enter bulk import input mode.
pub fn handle_start_bulk_import(state: &mut AppState) {
    state.bulk_import_mode = true;
    state.bulk_import_input.clear();
    state.set_status("DOI/arXiv ID 입력 (한 줄에 하나, Enter 제출, Esc 취소)");
}

/// Process submitted bulk import text (async).
/// Each line is a DOI or arXiv ID.
pub fn handle_bulk_import_submitted(state: &mut AppState, input: String) {
    state.bulk_import_mode = false;
    state.bulk_import_input.clear();
    state.start_processing("일괄 가져오는 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();
    let _mode = state.config.api_mode.clone();

    tokio::spawn(async move {
        let lines: Vec<String> = input
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let total = lines.len();
        let mut success = 0usize;
        let mut fail = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut dup_warnings: Vec<String> = Vec::new();

        for line in &lines {
            // Determine if it's a DOI or arXiv ID
            let is_arxiv = line.starts_with("arXiv:")
                || line.len() <= 20
                    && (line.contains('.')
                        && line
                            .split('.')
                            .next()
                            .map(|p| p.len() == 4)
                            .unwrap_or(false));

            let meta_result = if is_arxiv {
                fetch_arxiv_meta(&line).await
            } else {
                fetch_doi_meta(&line).await
            };

            match meta_result {
                Ok(meta) => {
                    let doc = pdf::RawMetadata {
                        title: meta.title.clone(),
                        authors: meta.authors.clone(),
                        journal: meta.journal.clone(),
                        pub_year: meta.pub_year,
                        doi: meta.doi.clone(),
                        arxiv_id: meta.arxiv_id.clone(),
                        abstract_text: meta.abstract_text.clone(),
                        ..Default::default()
                    };

                    let insert_result = {
                        let conn = match db.lock() {
                            Ok(c) => c,
                            Err(_) => {
                                fail += 1;
                                errors.push("DB 잠금 실패".to_string());
                                continue;
                            }
                        };
                        let new_doc = documents::Document {
                            title: doc.title.unwrap_or_else(|| "제목 없음".to_string()),
                            authors: if doc.authors.is_empty() {
                                None
                            } else {
                                Some(doc.authors.join(" and "))
                            },
                            journal: doc.journal,
                            pub_year: doc.pub_year,
                            doi: doc.doi,
                            arxiv_id: doc.arxiv_id,
                            abstract_text: doc.abstract_text,
                            source: Some("bulk_import".to_string()),
                            ..Default::default()
                        };
                        if let Ok(dups) = documents::find_duplicates(&conn, &new_doc)
                            && !dups.is_empty()
                        {
                            dup_warnings.push(format!("{}: 유사 문헌 {}건", line, dups.len()));
                        }
                        let result = documents::insert(&conn, &new_doc);
                        if let Ok(id) = result
                            && let Some(ref body) = doc.body_text
                        {
                            let _ = crate::db::documents_body::store(&conn, id, body);
                        }
                        result
                    };

                    match insert_result {
                        Ok(_) => success += 1,
                        Err(e) => {
                            fail += 1;
                            errors.push(format!("{}: {}", line, e));
                        }
                    }
                }
                Err(e) => {
                    fail += 1;
                    errors.push(format!("{}: {}", line, e));
                }
            }

            // Rate limiting: small delay between requests
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        let message = if errors.is_empty() && dup_warnings.is_empty() {
            format!("{}/{} 건 가져오기 완료", success, total)
        } else if dup_warnings.is_empty() {
            format!(
                "{}/{} 건 성공, {} 건 실패: {}",
                success,
                total,
                fail,
                errors.first().map(|s| s.as_str()).unwrap_or("")
            )
        } else {
            format!(
                "{}/{} 건 성공, {} 건 실패, 유사 문헌 경고: {}",
                success,
                total,
                fail,
                dup_warnings.join("; ")
            )
        };

        let _ = tx
            .send(AppAction::BulkImportResult {
                success_count: success,
                fail_count: fail,
                message,
            })
            .await;
    });
}

/// Show bulk import result.
pub fn handle_bulk_import_result(
    state: &mut AppState,
    _success: usize,
    _fail: usize,
    message: String,
) {
    state.finish_processing(&message);
    state.reload_documents();
}

async fn fetch_doi_meta(doi: &str) -> Result<pdf::RawMetadata, String> {
    let client = api::crossref::create_polite_http_client(None).map_err(|e| e.to_string())?;
    let body = api::crossref::fetch_by_doi(&client, doi)
        .await
        .map_err(|e| e.to_string())?;
    api_metadata::parse_crossref_response(&body).ok_or_else(|| "CrossRef 파싱 실패".to_string())
}

async fn fetch_arxiv_meta(arxiv_id: &str) -> Result<pdf::RawMetadata, String> {
    let id = arxiv_id.strip_prefix("arXiv:").unwrap_or(arxiv_id);
    let client = api::arxiv::create_client().map_err(|e| e.to_string())?;
    let body = api::arxiv::fetch_by_arxiv_id(&client, id)
        .await
        .map_err(|e| e.to_string())?;
    api_metadata::parse_arxiv_response(&body).ok_or_else(|| "arXiv 파싱 실패".to_string())
}
