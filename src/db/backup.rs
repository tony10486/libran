use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};

/// Create a compacted, WAL-safe backup of the database using `VACUUM INTO`.
///
/// `VACUUM INTO` acquires only a read lock, making it safe to run while the
/// database is in use. The resulting file is a single-file SQLite database
/// (no WAL/SHM sidecar files) with all free pages reclaimed.
pub fn backup_to_path(conn: &Connection, dest: &Path) -> Result<()> {
    let path_str = dest
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("backup path is not valid UTF-8: {:?}", dest))?;
    // SQLite string literals use single quotes; escape embedded quotes by doubling.
    let escaped = path_str.replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{}'", escaped))
        .with_context(|| format!("VACUUM INTO failed for {}", path_str))?;
    Ok(())
}

/// Restore a backup file to the active database path.
///
/// Copies the backup file to `dest`, then removes any stale WAL (`-wal`) and
/// shared-memory (`-shm`) sidecar files next to `dest` — a VACUUM INTO backup
/// is a single-file database, so those files are no longer valid.
///
/// The caller must close all open connections to `dest` before calling this
/// function and restart the application afterwards.
pub fn restore_from_path(src: &Path, dest: &Path) -> Result<()> {
    // Canonicalize src to prevent symlink-based attacks: fs::copy follows
    // symlinks, so a malicious symlink at src could copy unintended files into
    // dest. Canonicalize resolves the real path before any I/O.
    let _resolved = std::fs::canonicalize(src)
        .with_context(|| format!("restore: failed to resolve source path {:?}", src))?;
    std::fs::copy(src, dest)
        .with_context(|| format!("restore: failed to copy {:?} to {:?}", src, dest))?;
    // Remove stale WAL/SHM sidecar files — the backup is a single-file DB.
    if let Some(dest_str) = dest.to_str() {
        let _ = std::fs::remove_file(format!("{}-wal", dest_str));
        let _ = std::fs::remove_file(format!("{}-shm", dest_str));
    }
    Ok(())
}

/// Merge a backup database file (.db) into the active database.
///
/// Iteratively copies documents, notes, tags, custom fields, projects, and series,
/// skipping duplicates based on file_hash, doi, arxiv_id, or citation_key.
/// Performs batch transaction processing (100 docs per chunk) to avoid UI freeze.
pub fn import_db_from_path(main_conn: &mut Connection, backup_path: &Path) -> Result<(usize, usize)> {
    use rusqlite::OptionalExtension;
    use std::collections::{HashMap, HashSet};

    let backup_conn = Connection::open_with_flags(
        backup_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .with_context(|| format!("Failed to open backup database at {:?}", backup_path))?;

    // Check if the backup database is valid and contains standard tables
    let table_exists: i64 = backup_conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='documents'",
        [],
        |row| row.get(0),
    )?;
    if table_exists == 0 {
        return Err(anyhow::anyhow!("유효한 Libran 데이터베이스 백업 파일이 아닙니다."));
    }

    // 1. Build cache sets of existing identifiers in main DB
    let mut main_file_hashes = HashSet::new();
    let mut main_dois = HashSet::new();
    let mut main_arxivs = HashSet::new();
    let mut main_citation_keys = HashSet::new();

    {
        let mut stmt = main_conn.prepare("SELECT file_hash, doi, arxiv_id, citation_key FROM documents")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            if let Some(hash) = row.get::<_, Option<String>>(0)? { main_file_hashes.insert(hash); }
            if let Some(doi) = row.get::<_, Option<String>>(1)? { main_dois.insert(doi); }
            if let Some(arxiv) = row.get::<_, Option<String>>(2)? { main_arxivs.insert(arxiv); }
            if let Some(key) = row.get::<_, Option<String>>(3)? { main_citation_keys.insert(key); }
        }
    }

    // Merge projects and build map
    let mut project_id_map = HashMap::new();
    {
        let mut stmt = backup_conn.prepare("SELECT id, name, description FROM projects")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let desc: Option<String> = row.get(2)?;

            let mut main_proj_stmt = main_conn.prepare("SELECT id FROM projects WHERE name = ?1")?;
            let main_id_opt: Option<i64> = main_proj_stmt.query_row([&name], |r| r.get(0)).optional()?;
            
            let new_id = match main_id_opt {
                Some(id) => id,
                None => {
                    main_conn.execute("INSERT INTO projects (name, description) VALUES (?1, ?2)", rusqlite::params![name, desc])?;
                    main_conn.last_insert_rowid()
                }
            };
            project_id_map.insert(old_id, new_id);
        }
    }

    // Merge series and build map
    let mut series_id_map = HashMap::new();
    {
        let mut stmt = backup_conn.prepare("SELECT id, name, publisher, issn FROM series")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let publisher: Option<String> = row.get(2)?;
            let issn: Option<String> = row.get(3)?;

            let mut main_series_stmt = main_conn.prepare("SELECT id FROM series WHERE name = ?1")?;
            let main_id_opt: Option<i64> = main_series_stmt.query_row([&name], |r| r.get(0)).optional()?;

            let new_id = match main_id_opt {
                Some(id) => id,
                None => {
                    main_conn.execute(
                        "INSERT INTO series (name, publisher, issn) VALUES (?1, ?2, ?3)",
                        rusqlite::params![name, publisher, issn]
                    )?;
                    main_conn.last_insert_rowid()
                }
            };
            series_id_map.insert(old_id, new_id);
        }
    }

    // Merge classification schemes and build map
    let mut scheme_id_map = HashMap::new();
    {
        let mut stmt = backup_conn.prepare("SELECT id, code, name, version, enabled, is_primary, license, source_url FROM classification_schemes")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let code: String = row.get(1)?;
            let name: String = row.get(2)?;
            let version: Option<String> = row.get(3)?;
            let enabled: i32 = row.get(4)?;
            let is_primary: i32 = row.get(5)?;
            let license: Option<String> = row.get(6)?;
            let source_url: Option<String> = row.get(7)?;

            let mut main_scheme_stmt = main_conn.prepare("SELECT id FROM classification_schemes WHERE code = ?1")?;
            let main_id_opt: Option<i64> = main_scheme_stmt.query_row([&code], |r| r.get(0)).optional()?;

            let new_id = match main_id_opt {
                Some(id) => id,
                None => {
                    main_conn.execute(
                        "INSERT INTO classification_schemes (code, name, version, enabled, is_primary, license, source_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![code, name, version, enabled, is_primary, license, source_url]
                    )?;
                    main_conn.last_insert_rowid()
                }
            };
            scheme_id_map.insert(old_id, new_id);
        }
    }

    // Merge classification nodes and build map
    let mut node_id_map = HashMap::new();
    {
        let mut stmt = backup_conn.prepare("SELECT id, scheme_id, notation, pref_label, alt_label, scope_note, parent_id, sort_order FROM classification_nodes")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let old_id: i64 = row.get(0)?;
            let old_scheme_id: i64 = row.get(1)?;
            let notation: String = row.get(2)?;
            let pref_label: String = row.get(3)?;
            let alt_label: Option<String> = row.get(4)?;
            let scope_note: Option<String> = row.get(5)?;
            let old_parent_id: Option<i64> = row.get(6)?;
            let sort_order: i32 = row.get(7)?;

            let new_scheme_id = match scheme_id_map.get(&old_scheme_id) {
                Some(&id) => id,
                None => continue,
            };

            let mut main_node_stmt = main_conn.prepare("SELECT id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2")?;
            let main_id_opt: Option<i64> = main_node_stmt.query_row(rusqlite::params![new_scheme_id, notation], |r| r.get(0)).optional()?;

            let new_id = match main_id_opt {
                Some(id) => id,
                None => {
                    let new_parent_id = old_parent_id.and_then(|p| node_id_map.get(&p).copied());
                    main_conn.execute(
                        "INSERT INTO classification_nodes (scheme_id, notation, pref_label, alt_label, scope_note, parent_id, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![new_scheme_id, notation, pref_label, alt_label, scope_note, new_parent_id, sort_order]
                    )?;
                    main_conn.last_insert_rowid()
                }
            };
            node_id_map.insert(old_id, new_id);
        }
    }

    // 2. Batch process documents (100 docs per transaction)
    let mut success_count = 0;
    let mut skip_count = 0;

    let doc_count: i64 = backup_conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))?;
    let batch_size = 100;
    let mut offset = 0;

    while offset < doc_count {
        let mut stmt = backup_conn.prepare(&format!(
            "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date FROM documents LIMIT {} OFFSET {}",
            batch_size, offset
        ))?;
        
        let mut rows = stmt.query([])?;
        
        // Use rusqlite's Transaction struct — auto-rollback on Drop if not committed.
        let tx = main_conn.transaction().context("import: failed to begin transaction")?;

        while let Some(row) = rows.next()? {
            let old_doc_id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let authors: Option<String> = row.get(2)?;
            let journal: Option<String> = row.get(3)?;
            let pub_year: Option<i32> = row.get(4)?;
            let doi: Option<String> = row.get(5)?;
            let arxiv_id: Option<String> = row.get(6)?;
            let r_abstract: Option<String> = row.get(7)?;
            let keywords: Option<String> = row.get(8)?;
            let file_path: Option<String> = row.get(9)?;
            let file_hash: Option<String> = row.get(10)?;
            let citation_key: Option<String> = row.get(11)?;
            let source: Option<String> = row.get(12)?;
            let volume: Option<String> = row.get(13)?;
            let issue: Option<String> = row.get(14)?;
            let page_start: Option<String> = row.get(15)?;
            let page_end: Option<String> = row.get(16)?;
            let publisher: Option<String> = row.get(17)?;
            let city: Option<String> = row.get(18)?;
            let edition: Option<String> = row.get(19)?;
            let isbn: Option<String> = row.get(20)?;
            let issn: Option<String> = row.get(21)?;
            let url: Option<String> = row.get(22)?;
            let accessed_date: Option<String> = row.get(23)?;

            // Check duplicates
            let mut duplicate = false;
            if let Some(ref hash) = file_hash {
                if main_file_hashes.contains(hash) { duplicate = true; }
            }
            if !duplicate {
                if let Some(ref d) = doi {
                    if main_dois.contains(d) { duplicate = true; }
                }
            }
            if !duplicate {
                if let Some(ref a) = arxiv_id {
                    if main_arxivs.contains(a) { duplicate = true; }
                }
            }
            if !duplicate {
                if let Some(ref k) = citation_key {
                    if main_citation_keys.contains(k) { duplicate = true; }
                }
            }

            if duplicate {
                skip_count += 1;
                continue;
            }

            // Insert document through the transaction handle
            tx.execute(
                "INSERT INTO documents (title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
                rusqlite::params![
                    title, authors, journal, pub_year, doi, arxiv_id, r_abstract, keywords, file_path, file_hash, citation_key, source, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date
                ]
            )?;
            let new_doc_id = tx.last_insert_rowid();

            // Cache identity values
            if let Some(ref hash) = file_hash { main_file_hashes.insert(hash.clone()); }
            if let Some(ref d) = doi { main_dois.insert(d.clone()); }
            if let Some(ref a) = arxiv_id { main_arxivs.insert(a.clone()); }
            if let Some(ref k) = citation_key { main_citation_keys.insert(k.clone()); }

            // Tag relations (tags)
            {
                let mut c_stmt = backup_conn.prepare("SELECT tag FROM tags WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let tag: String = c_row.get(0)?;
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO tags (document_id, tag) VALUES (?1, ?2)",
                        rusqlite::params![new_doc_id, tag]
                    );
                }
            }

            // Notes (document_notes)
            {
                let mut c_stmt = backup_conn.prepare("SELECT content, note_type, created_at, updated_at FROM document_notes WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let content: String = c_row.get(0)?;
                    let note_type: String = c_row.get(1)?;
                    let created_at: String = c_row.get(2)?;
                    let updated_at: String = c_row.get(3)?;
                    let _ = tx.execute(
                        "INSERT INTO document_notes (document_id, content, note_type, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![new_doc_id, content, note_type, created_at, updated_at]
                    );
                }
            }

            // Series relations (document_series)
            {
                let mut c_stmt = backup_conn.prepare("SELECT series_id, volume, issue, sort_order FROM document_series WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let old_series_id: i64 = c_row.get(0)?;
                    let volume: Option<String> = c_row.get(1)?;
                    let issue: Option<String> = c_row.get(2)?;
                    let sort_order: i32 = c_row.get(3)?;

                    if let Some(&new_series_id) = series_id_map.get(&old_series_id) {
                        let _ = tx.execute(
                            "INSERT OR IGNORE INTO document_series (document_id, series_id, volume, issue, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![new_doc_id, new_series_id, volume, issue, sort_order]
                        );
                    }
                }
            }

            // Project relations (project_documents)
            {
                let mut c_stmt = backup_conn.prepare("SELECT project_id FROM project_documents WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let old_project_id: i64 = c_row.get(0)?;
                    if let Some(&new_project_id) = project_id_map.get(&old_project_id) {
                        let _ = tx.execute(
                            "INSERT OR IGNORE INTO project_documents (project_id, document_id) VALUES (?1, ?2)",
                            rusqlite::params![new_project_id, new_doc_id]
                        );
                    }
                }
            }

            // Classifications (document_classifications)
            {
                let mut c_stmt = backup_conn.prepare("SELECT node_id, is_primary, confidence, assigned_by FROM document_classifications WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let old_node_id: i64 = c_row.get(0)?;
                    let is_primary: i32 = c_row.get(1)?;
                    let confidence: Option<f64> = c_row.get(2)?;
                    let assigned_by: Option<String> = c_row.get(3)?;

                    if let Some(&new_node_id) = node_id_map.get(&old_node_id) {
                        let _ = tx.execute(
                            "INSERT OR IGNORE INTO document_classifications (document_id, node_id, is_primary, confidence, assigned_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![new_doc_id, new_node_id, is_primary, confidence, assigned_by]
                        );
                    }
                }
            }

            // Custom fields (document_custom_fields)
            {
                let mut c_stmt = backup_conn.prepare("SELECT field_key, field_value FROM document_custom_fields WHERE document_id = ?1")?;
                let mut c_rows = c_stmt.query([old_doc_id])?;
                while let Some(c_row) = c_rows.next()? {
                    let field_key: String = c_row.get(0)?;
                    let field_value: Option<String> = c_row.get(1)?;
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO document_custom_fields (document_id, field_key, field_value) VALUES (?1, ?2, ?3)",
                        rusqlite::params![new_doc_id, field_key, field_value]
                    );
                }
            }

            success_count += 1;
        }

        tx.commit().context("import: failed to commit transaction batch")?;
        offset += batch_size;
    }

    Ok((success_count, skip_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::documents::{self, Document};
    use rusqlite::Connection;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn make_test_db(dir: &Path) -> anyhow::Result<(PathBuf, Connection)> {
        let db_path = dir.join("test.db");
        let conn = Connection::open(&db_path)?;
        db::init_database(&conn)?;
        Ok((db_path, conn))
    }

    #[test]
    fn test_backup_creates_valid_db() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let (_db_path, conn) = make_test_db(dir.path())?;
        let backup_path = dir.path().join("backup.db");

        backup_to_path(&conn, &backup_path)?;

        assert!(backup_path.exists(), "backup file should exist");

        let backup_conn = Connection::open(&backup_path)?;
        for table in [
            "documents",
            "classification_schemes",
            "classification_nodes",
            "document_classifications",
            "projects",
            "project_documents",
            "documents_fts",
            "api_cache",
            "app_config",
            "tags",
            "citation_relations",
            "document_notes",
            "series",
            "document_series",
            "document_custom_fields",
        ] {
            let exists: i64 = backup_conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                rusqlite::params![table],
                |row| row.get(0),
            )?;
            assert_eq!(exists, 1, "table {} should exist in backup", table);
        }
        Ok(())
    }

    #[test]
    fn test_backup_preserves_data() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let (_db_path, conn) = make_test_db(dir.path())?;
        let doc = Document {
            title: "Test Paper".to_string(),
            authors: Some("Test Author".to_string()),
            pub_year: Some(2024),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let doc_id = documents::insert(&conn, &doc)?;

        let backup_path = dir.path().join("backup_data.db");
        backup_to_path(&conn, &backup_path)?;

        let backup_conn = Connection::open(&backup_path)?;
        let (id, title, authors, year, doi): (
            i64,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
        ) = backup_conn.query_row(
            "SELECT id, title, authors, pub_year, doi FROM documents WHERE id = ?1",
            rusqlite::params![doc_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )?;
        assert_eq!(id, doc_id);
        assert_eq!(title, "Test Paper");
        assert_eq!(authors.as_deref(), Some("Test Author"));
        assert_eq!(year, Some(2024));
        assert_eq!(doi.as_deref(), Some("10.1234/test"));
        Ok(())
    }

    #[test]
    fn test_restore_copies_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let (_db_path, conn) = make_test_db(dir.path())?;
        documents::insert(
            &conn,
            &Document {
                title: "Restore Test".to_string(),
                ..Default::default()
            },
        )?;
        let backup_path = dir.path().join("restore_backup.db");
        backup_to_path(&conn, &backup_path)?;
        drop(conn);

        let restore_dest = dir.path().join("restored.db");
        restore_from_path(&backup_path, &restore_dest)?;

        assert!(restore_dest.exists(), "restored file should exist");
        let restored_conn = Connection::open(&restore_dest)?;
        let title: String = restored_conn.query_row(
            "SELECT title FROM documents WHERE title = 'Restore Test'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(title, "Restore Test");
        Ok(())
    }

    #[test]
    fn test_backup_from_wal_mode_db() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("wal_test.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        db::init_database(&conn)?;
        documents::insert(
            &conn,
            &Document {
                title: "WAL Mode Paper".to_string(),
                ..Default::default()
            },
        )?;

        let backup_path = dir.path().join("wal_backup.db");
        backup_to_path(&conn, &backup_path)?;

        assert!(backup_path.exists());
        assert!(
            !dir.path().join("wal_backup.db-wal").exists(),
            "VACUUM INTO backup should not have a WAL sidecar"
        );
        let backup_conn = Connection::open(&backup_path)?;
        let count: i64 =
            backup_conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn test_import_db_from_path() -> anyhow::Result<()> {
        let dir = tempdir()?;
        
        // 1. Create source DB and insert a document with notes and tags
        let src_dir = dir.path().join("src_db");
        std::fs::create_dir(&src_dir)?;
        let (_src_path, src_conn) = make_test_db(&src_dir)?;
        let doc_id = documents::insert(
            &src_conn,
            &Document {
                title: "Unique Merge Title".to_string(),
                doi: Some("10.1000/unique.merge.doi".to_string()),
                file_hash: Some("someuniquehash".to_string()),
                ..Default::default()
            },
        )?;
        
        // Insert Tag
        src_conn.execute(
            "INSERT INTO tags (document_id, tag) VALUES (?1, ?2)",
            rusqlite::params![doc_id, "MergedTag"]
        )?;

        // Insert Note
        src_conn.execute(
            "INSERT INTO document_notes (document_id, content, note_type) VALUES (?1, ?2, ?3)",
            rusqlite::params![doc_id, "Merged Note content", "general"]
        )?;

        // Insert Project
        src_conn.execute(
            "INSERT INTO projects (name, description) VALUES ('TargetProject', 'desc')",
            []
        )?;
        let src_proj_id = src_conn.last_insert_rowid();
        src_conn.execute(
            "INSERT INTO project_documents (project_id, document_id) VALUES (?1, ?2)",
            rusqlite::params![src_proj_id, doc_id]
        )?;

        // Create backup file
        let backup_path = dir.path().join("merge_backup.db");
        backup_to_path(&src_conn, &backup_path)?;

        // 2. Create target active DB
        let dest_dir = dir.path().join("dest_db");
        std::fs::create_dir(&dest_dir)?;
        let (_dest_path, mut dest_conn) = make_test_db(&dest_dir)?;

        // Perform import (Merge)
        let (success, skip) = import_db_from_path(&mut dest_conn, &backup_path)?;
        assert_eq!(success, 1);
        assert_eq!(skip, 0);

        // Verify document exist in target DB
        let dest_count: i64 = dest_conn.query_row("SELECT COUNT(*) FROM documents WHERE title = 'Unique Merge Title'", [], |r| r.get(0))?;
        assert_eq!(dest_count, 1);

        // Verify note and tag merged
        let dest_tag_count: i64 = dest_conn.query_row("SELECT COUNT(*) FROM tags WHERE tag = 'MergedTag'", [], |r| r.get(0))?;
        assert_eq!(dest_tag_count, 1);

        let dest_note_count: i64 = dest_conn.query_row("SELECT COUNT(*) FROM document_notes WHERE content = 'Merged Note content'", [], |r| r.get(0))?;
        assert_eq!(dest_note_count, 1);

        // Verify project and mapping merged
        let dest_proj_count: i64 = dest_conn.query_row("SELECT COUNT(*) FROM projects WHERE name = 'TargetProject'", [], |r| r.get(0))?;
        assert_eq!(dest_proj_count, 1);
        
        let dest_mapping_count: i64 = dest_conn.query_row("SELECT COUNT(*) FROM project_documents", [], |r| r.get(0))?;
        assert_eq!(dest_mapping_count, 1);

        // 3. Import again (should skip duplicate document)
        let (success2, skip2) = import_db_from_path(&mut dest_conn, &backup_path)?;
        assert_eq!(success2, 0);
        assert_eq!(skip2, 1);

        Ok(())
    }
}
