use crate::app::action::AppAction;
use crate::app::AppState;
use crate::citation::bibtex_parser::{self, ParsedEntry};
use crate::db::documents;
use crate::db::fts_query::normalize_nfc;

use std::path::Path;

/// Enter file import input mode.
pub fn handle_start_file_import(state: &mut AppState) {
    state.file_import_mode = true;
    state.file_import_input.clear();
    state.set_status("BibTeX 파일 경로 입력 (Enter 제출, Esc 취소)");
}

/// Process submitted BibTeX file (async).
pub fn handle_file_import_submitted(state: &mut AppState, path_str: String) {
    state.file_import_mode = false;
    state.file_import_input.clear();
    state.start_processing("BibTeX 가져오는 중...");

    let db = state.db.clone();
    let tx = state.action_tx.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || -> Result<usize, String> {
            let path = Path::new(&path_str);
            if !path.exists() {
                return Err(format!("파일 없음: {}", path_str));
            }
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("파일 읽기 실패: {}", e))?;
            let entries = bibtex_parser::parse_bibtex(&content)
                .map_err(|e| format!("파싱 실패: {}", e))?;

            let conn = db.lock().map_err(|e| e.to_string())?;
            let mut count = 0;
            for entry in &entries {
                let doc = entry_to_document(entry);
                if documents::insert(&conn, &doc).is_ok() {
                    count += 1;
                }
            }
            Ok(count)
        })
        .await;

        let action = match result {
            Ok(Ok(count)) => AppAction::FileImportResult {
                count,
                message: format!("{} 건 가져오기 완료", count),
            },
            Ok(Err(m)) => AppAction::FileImportResult {
                count: 0,
                message: format!("가져오기 실패: {}", m),
            },
            Err(e) => AppAction::FileImportResult {
                count: 0,
                message: format!("태스크 실패: {}", e),
            },
        };
        let _ = tx.send(action).await;
    });
}

/// Show file import result.
pub fn handle_file_import_result(state: &mut AppState, _count: usize, message: String) {
    state.finish_processing(&message);
    state.reload_documents();
}

fn entry_to_document(entry: &ParsedEntry) -> documents::Document {
    let get = |key: &str| entry.fields.get(key).cloned();

    let authors = get("author");
    let title = get("title").unwrap_or_else(|| "제목 없음".to_string());
    let journal = get("journal").or_else(|| get("booktitle"));
    let year = get("year").and_then(|y| y.parse::<i64>().ok());
    let doi = get("doi");
    let arxiv_id = get("eprint").filter(|_| {
        entry
            .fields
            .get("archiveprefix")
            .map(|p| p.eq_ignore_ascii_case("arxiv"))
            .unwrap_or(true)
    });
    let keywords = get("keywords");
    let volume = get("volume");
    let issue = get("number");
    let page_start = get("pages").and_then(|p| {
        p.split("--")
            .next()
            .map(|s| s.trim().to_string())
    });
    let publisher = get("publisher");
    let url = get("url");
    let isbn = get("isbn");
    let issn = get("issn");

    documents::Document {
        title: normalize_nfc(&title),
        authors: authors.map(|a| normalize_nfc(&a)),
        journal: journal.map(|j| normalize_nfc(&j)),
        pub_year: year,
        doi,
        arxiv_id,
        abstract_text: None,
        keywords,
        file_path: None,
        file_hash: None,
        citation_key: Some(entry.citation_key.clone()),
        source: Some(format!("bibtex:{}", entry.entry_type)),
        volume,
        issue,
        page_start,
        publisher,
        url,
        isbn,
        issn,
        ..Default::default()
    }
}
