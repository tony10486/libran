use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

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
    std::fs::copy(src, dest)
        .with_context(|| format!("restore: failed to copy {:?} to {:?}", src, dest))?;
    // Remove stale WAL/SHM sidecar files — the backup is a single-file DB.
    if let Some(dest_str) = dest.to_str() {
        let _ = std::fs::remove_file(format!("{}-wal", dest_str));
        let _ = std::fs::remove_file(format!("{}-shm", dest_str));
    }
    Ok(())
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
}
