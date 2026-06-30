use anyhow::{Result, anyhow};
use directories::BaseDirs;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SioyekHighlight {
    pub page: i64,
    pub text: String,
    pub color_type: String,
}

/// Sioyek 데이터베이스 폴더 경로를 감지합니다.
pub fn get_sioyek_dir() -> Option<PathBuf> {
    let base_dirs = BaseDirs::new()?;
    let home = base_dirs.home_dir();

    // OS별 디렉토리 우선순위 탐색
    #[cfg(target_os = "macos")]
    {
        let paths = vec![
            home.join("Library/Application Support/sioyek"),
            home.join("Library/Application Support/Sioyek"),
            home.join(".config/sioyek"),
        ];
        for p in paths {
            if p.exists() {
                return Some(p);
            }
        }
        Some(home.join("Library/Application Support/sioyek"))
    }
    #[cfg(target_os = "windows")]
    {
        let local_appdata = base_dirs.data_local_dir();
        let appdata = base_dirs.data_dir();
        let paths = vec![
            local_appdata.join("sioyek"),
            appdata.join("sioyek"),
        ];
        for p in paths {
            if p.exists() {
                return Some(p);
            }
        }
        Some(local_appdata.join("sioyek"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux 등
    {
        let paths = vec![
            base_dirs.data_dir().join("sioyek"),
            home.join(".local/share/sioyek"),
            home.join(".config/sioyek"),
        ];
        for p in paths {
            if p.exists() {
                return Some(p);
            }
        }
        Some(home.join(".local/share/sioyek"))
    }
}

/// Sioyek의 local.db 및 shared.db 절대 경로를 가져옵니다.
pub fn get_sioyek_db_paths() -> Option<(PathBuf, PathBuf)> {
    let dir = get_sioyek_dir()?;
    let local_db = dir.join("local.db");
    let shared_db = dir.join("shared.db");
    Some((local_db, shared_db))
}

/// 시스템 PATH 및 표준 설치 경로에서 Sioyek 실행 파일을 탐색합니다.
pub fn find_sioyek_executable() -> Option<String> {
    // 1. 시스템 PATH 환경변수 내에서 검사
    if std::process::Command::new("sioyek")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        return Some("sioyek".to_string());
    }

    // 2. OS별 기본 설치 위치 하드코딩 탐색
    #[cfg(target_os = "macos")]
    {
        let paths = vec![
            "/Applications/Sioyek.app/Contents/MacOS/sioyek",
            "/Applications/sioyek.app/Contents/MacOS/sioyek",
            "~/Applications/Sioyek.app/Contents/MacOS/sioyek",
            "/opt/homebrew/bin/sioyek",
            "/usr/local/bin/sioyek",
        ];
        for p in paths {
            let expanded = if p.starts_with('~') {
                if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
                    home.join(&p[2..])
                } else {
                    PathBuf::from(p)
                }
            } else {
                PathBuf::from(p)
            };
            if expanded.exists() {
                return Some(expanded.to_string_lossy().to_string());
            }
        }

        // Caskroom 하위 버전의 Sioyek.app 내 실행파일 동적으로 찾기
        if let Ok(entries) = std::fs::read_dir("/opt/homebrew/Caskroom/sioyek") {
            for entry in entries.flatten() {
                let p = entry.path().join("sioyek.app").join("Contents").join("MacOS").join("sioyek");
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let paths = vec![
            "C:\\Program Files\\Sioyek\\sioyek.exe",
            "C:\\Program Files (x86)\\Sioyek\\sioyek.exe",
        ];
        for p in paths {
            let path = Path::new(p);
            if path.exists() {
                return Some(p.to_string());
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux 등
    {
        let paths = vec![
            "/usr/bin/sioyek",
            "/usr/local/bin/sioyek",
            "/snap/bin/sioyek",
        ];
        for p in paths {
            let path = Path::new(p);
            if path.exists() {
                return Some(p.to_string());
            }
        }
    }

    None
}

/// local.db에서 파일 경로에 맵핑된 MD5 해시값을 조회합니다.
pub fn get_document_hash(local_db_path: &Path, file_path: &str) -> Result<Option<String>> {
    if !local_db_path.exists() {
        return Err(anyhow!("Sioyek local.db를 찾을 수 없습니다: {:?}", local_db_path));
    }

    let conn = Connection::open(local_db_path)?;
    
    // Sioyek은 경로 구분자를 시스템 표준에 맞춰 저장하거나 정규화하여 관리합니다.
    // 절대경로 매칭을 시도하고 실패 시 파일 이름 기반 등의 대안도 고려할 수 있도록 쿼리합니다.
    let mut stmt = conn.prepare("SELECT hash FROM document_hash WHERE path = ?1")?;
    let mut rows = stmt.query(params![file_path])?;
    
    if let Some(row) = rows.next()? {
        let hash: String = row.get(0)?;
        Ok(Some(hash))
    } else {
        // 절대 경로에 대해 유연하게 검사하기 위해 부분 일치(LIKE) 백업 쿼리도 시도합니다.
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        
        if !file_name.is_empty() {
            let mut stmt_like = conn.prepare("SELECT hash, path FROM document_hash WHERE path LIKE ?1")?;
            let search_pattern = format!("%{}", file_name);
            let mut rows_like = stmt_like.query(params![search_pattern])?;
            if let Some(row_like) = rows_like.next()? {
                let hash: String = row_like.get(0)?;
                let matched_path: String = row_like.get(1)?;
                tracing::info!("Sioyek DB에서 파일 이름 부분 일치로 해시 검색 성공 (요청: {}, 매칭: {}, 해시: {})", file_path, matched_path, hash);
                return Ok(Some(hash));
            }
        }
        Ok(None)
    }
}

/// shared.db의 opened_books 테이블에서 마지막으로 읽은 페이지를 조회합니다. (0-indexed)
pub fn get_last_read_page(shared_db_path: &Path, doc_hash: &str) -> Result<Option<i64>> {
    if !shared_db_path.exists() {
        return Err(anyhow!("Sioyek shared.db를 찾을 수 없습니다: {:?}", shared_db_path));
    }

    let conn = Connection::open(shared_db_path)?;
    let mut stmt = conn.prepare("SELECT page FROM opened_books WHERE document_path = ?1")?;
    let mut rows = stmt.query(params![doc_hash])?;

    if let Some(row) = rows.next()? {
        let page: i64 = row.get(0)?;
        Ok(Some(page))
    } else {
        Ok(None)
    }
}

/// shared.db의 highlights 테이블에서 하이라이트 주석 데이터를 추출합니다.
pub fn get_highlights(shared_db_path: &Path, doc_hash: &str) -> Result<Vec<SioyekHighlight>> {
    if !shared_db_path.exists() {
        return Err(anyhow!("Sioyek shared.db를 찾을 수 없습니다: {:?}", shared_db_path));
    }

    let conn = Connection::open(shared_db_path)?;
    
    // highlights 테이블이 존재하는지 먼저 확인
    let table_exists: i32 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='highlights'",
        [],
        |row| row.get(0),
    )?;

    if table_exists == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT page, desc, type FROM highlights WHERE document_path = ?1 AND desc IS NOT NULL AND trim(desc) != ''"
    )?;
    
    let rows = stmt.query_map(params![doc_hash], |row| {
        Ok(SioyekHighlight {
            page: row.get(0)?,
            text: row.get(1)?,
            color_type: row.get(2).unwrap_or_else(|_| "default".to_string()),
        })
    })?;

    let mut highlights = Vec::new();
    for row in rows {
        if let Ok(h) = row {
            highlights.push(h);
        }
    }
    Ok(highlights)
}

/// Sioyek의 리더 데이터를 Libran의 DB와 동기화합니다.
pub fn sync_sioyek_data(libran_conn: &Connection, doc_id: i64) -> Result<String> {
    // 1. Libran DB에서 파일 경로 조회
    let file_path: Option<String> = libran_conn.query_row(
        "SELECT file_path FROM documents WHERE id = ?1",
        params![doc_id],
        |row| row.get(0),
    ).map_err(|e| anyhow!("Libran DB에서 문헌 정보를 조회할 수 없습니다: {}", e))?;

    let file_path = match file_path {
        Some(p) if !p.trim().is_empty() => p,
        _ => return Ok("파일 경로가 등록되지 않은 문헌이므로 동기화를 건너뜁니다.".to_string()),
    };

    // 2. Sioyek DB 경로 감지
    let (local_db_path, shared_db_path) = match get_sioyek_db_paths() {
        Some(paths) => paths,
        None => return Err(anyhow!("Sioyek 데이터베이스 폴더를 감지할 수 없습니다. Sioyek이 올바르게 설치되었는지 확인해 주세요.")),
    };

    if !local_db_path.exists() || !shared_db_path.exists() {
        return Ok("Sioyek 데이터베이스 파일이 존재하지 않습니다. 아직 문서를 읽지 않았을 수 있습니다.".to_string());
    }

    // 3. Sioyek local.db에서 파일 해시 조회
    let doc_hash = match get_document_hash(&local_db_path, &file_path)? {
        Some(h) => h,
        None => return Ok(format!("Sioyek 내역에서 해당 파일을 찾을 수 없습니다: {}", file_path)),
    };

    let mut updates = Vec::new();

    // 4. 읽기 진척도 동기화
    if let Some(page) = get_last_read_page(&shared_db_path, &doc_hash)? {
        let mut progress_pct = 0;
        let mut progress_msg = format!("읽기 진척도 동기화: {} 페이지", page + 1);

        if let Ok(doc) = lopdf::Document::load(&file_path) {
            let total_pages = doc.get_pages().len();
            if total_pages > 0 {
                let pct = (((page + 1) as f64 / total_pages as f64) * 100.0).round().min(100.0) as i64;
                progress_pct = pct;
                progress_msg = format!("읽기 진척도 동기화: {}% ({} / {} 페이지)", pct, page + 1, total_pages);
            }
        }

        let status = if progress_pct >= 100 { "read" } else { "reading" };
        libran_conn.execute(
            "UPDATE documents SET reading_progress = ?1, reading_status = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![progress_pct, status, doc_id],
        )?;
        updates.push(progress_msg);
    }

    // 5. 하이라이트 동기화 (중복 방지)
    let highlights = get_highlights(&shared_db_path, &doc_hash)?;
    let mut added_highlights_count = 0;

    // 이미 Libran에 등록된 하이라이트 데이터 수집
    let mut existing_notes = std::collections::HashSet::new();
    let mut stmt = libran_conn.prepare(
        "SELECT content FROM document_notes WHERE document_id = ?1 AND note_type = 'sioyek_highlight'"
    )?;
    let rows = stmt.query_map(params![doc_id], |row| row.get::<_, String>(0))?;
    for r in rows {
        if let Ok(c) = r {
            existing_notes.insert(c.trim().to_string());
        }
    }
    drop(stmt);

    for h in highlights {
        let content = h.text.trim().to_string();
        if content.is_empty() {
            continue;
        }

        // 중복되지 않은 새로운 하이라이트 내용만 추가
        if !existing_notes.contains(&content) {
            let note_content = content.clone();
            // document_notes에 삽입
            libran_conn.execute(
                "INSERT INTO document_notes (document_id, content, note_type, created_at, updated_at)
                 VALUES (?1, ?2, 'sioyek_highlight', datetime('now'), datetime('now'))",
                params![doc_id, note_content],
            )?;
            existing_notes.insert(content);
            added_highlights_count += 1;
        }
    }

    if added_highlights_count > 0 {
        updates.push(format!("새로운 하이라이트 {}개 추가", added_highlights_count));
    }

    if updates.is_empty() {
        Ok("가져올 새로운 Sioyek 데이터가 없습니다.".to_string())
    } else {
        Ok(format!("동기화 완료 ({})", updates.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_mock_local_db(db_path: &Path, file_path: &str, expected_hash: &str) {
        let conn = Connection::open(db_path).unwrap();
        conn.execute(
            "CREATE TABLE document_hash (path TEXT PRIMARY KEY, hash TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO document_hash (path, hash) VALUES (?1, ?2)",
            params![file_path, expected_hash],
        )
        .unwrap();
    }

    fn setup_mock_shared_db(db_path: &Path, doc_hash: &str, page: i64, highlights: &[(&str, &str)]) {
        let conn = Connection::open(db_path).unwrap();
        conn.execute(
            "CREATE TABLE opened_books (document_path TEXT PRIMARY KEY, page INTEGER, offset_x REAL, offset_y REAL, zoom REAL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO opened_books (document_path, page, offset_x, offset_y, zoom) VALUES (?1, ?2, 0.0, 0.0, 1.0)",
            params![doc_hash, page],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE highlights (id INTEGER PRIMARY KEY, document_path TEXT, page INTEGER, type TEXT, desc TEXT)",
            [],
        )
        .unwrap();

        for (text, h_type) in highlights {
            conn.execute(
                "INSERT INTO highlights (document_path, page, type, desc) VALUES (?1, ?2, ?3, ?4)",
                params![doc_hash, page, h_type, text],
            )
            .unwrap();
        }
    }


    #[test]
    fn test_get_document_hash() {
        let tmp_dir = tempdir().unwrap();
        let local_db = tmp_dir.path().join("local.db");
        let file_path = "/path/to/my/document.pdf";
        let expected_hash = "1a2b3c4d5e6f";

        setup_mock_local_db(&local_db, file_path, expected_hash);

        let hash = get_document_hash(&local_db, file_path).unwrap();
        assert_eq!(hash, Some(expected_hash.to_string()));

        // 부분 매칭 테스트 (파일 이름 기반)
        let hash_partial = get_document_hash(&local_db, "/another/different/path/document.pdf").unwrap();
        assert_eq!(hash_partial, Some(expected_hash.to_string()));

        // 없는 파일 매칭
        let hash_none = get_document_hash(&local_db, "/path/to/nonexistent.pdf").unwrap();
        assert_eq!(hash_none, None);
    }

    #[test]
    fn test_get_last_read_page() {
        let tmp_dir = tempdir().unwrap();
        let shared_db = tmp_dir.path().join("shared.db");
        let doc_hash = "1a2b3c4d5e6f";
        let expected_page = 14;

        setup_mock_shared_db(&shared_db, doc_hash, expected_page, &[]);

        let page = get_last_read_page(&shared_db, doc_hash).unwrap();
        assert_eq!(page, Some(expected_page));
    }

    #[test]
    fn test_get_highlights() {
        let tmp_dir = tempdir().unwrap();
        let shared_db = tmp_dir.path().join("shared.db");
        let doc_hash = "1a2b3c4d5e6f";
        let raw_highlights = vec![
            ("This is an important sentence.", "yellow"),
            ("Another key discovery.", "red"),
        ];

        setup_mock_shared_db(&shared_db, doc_hash, 5, &raw_highlights);

        let highlights = get_highlights(&shared_db, doc_hash).unwrap();
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].text, "This is an important sentence.");
        assert_eq!(highlights[0].color_type, "yellow");
        assert_eq!(highlights[0].page, 5);
    }
}
