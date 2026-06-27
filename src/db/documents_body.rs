use anyhow::Result;
use rusqlite::{Connection, params};

use crate::db::fts_query::{SearchPlan, build_search_plan, escape_like, normalize_nfc};

/// Store body text for a document, replacing any existing body text.
/// The FTS index is kept in sync via triggers on `documents_body`.
pub fn store(conn: &Connection, doc_id: i64, body_text: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO documents_body (document_id, body_text) VALUES (?1, ?2)",
        params![doc_id, normalize_nfc(body_text)],
    )?;
    Ok(())
}

/// Retrieve body text for a document, if stored.
pub fn get(conn: &Connection, doc_id: i64) -> Result<Option<String>> {
    let result: std::result::Result<String, rusqlite::Error> = conn.query_row(
        "SELECT body_text FROM documents_body WHERE document_id = ?1",
        params![doc_id],
        |row| row.get(0),
    );
    match result {
        Ok(text) => Ok(Some(text)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Search body text via the FTS5 trigram index. Returns matching document IDs.
pub fn search_body(conn: &Connection, term: &str) -> Result<Vec<i64>> {
    let term = term.trim();
    if term.is_empty() {
        return Ok(Vec::new());
    }

    let term = normalize_nfc(term);

    match build_search_plan(&term) {
        SearchPlan::FtsMatch(escaped) => search_body_fts(conn, &escaped),
        _ => search_body_like(conn, &term),
    }
}

fn search_body_fts(conn: &Connection, escaped: &str) -> Result<Vec<i64>> {
    let mut stmt =
        conn.prepare("SELECT rowid FROM documents_body_fts WHERE documents_body_fts MATCH ?1")?;
    let rows = stmt.query_map(params![escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_body_like(conn: &Connection, term: &str) -> Result<Vec<i64>> {
    let pattern = format!("%{}%", escape_like(term));
    let mut stmt =
        conn.prepare("SELECT document_id FROM documents_body WHERE body_text LIKE ?1 ESCAPE '\\'")?;
    let rows = stmt.query_map(params![pattern], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::documents::Document;
    use rusqlite::Connection;

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        db::init_database(&conn)?;
        Ok(conn)
    }

    fn make_doc(title: &str) -> Document {
        Document {
            title: title.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_body_stored() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Body Test Paper"))?;

        let body = "This is the full body text of the paper discussing quantum entanglement.";
        store(&conn, doc_id, body)?;

        let retrieved = get(&conn, doc_id)?;
        assert_eq!(retrieved.as_deref(), Some(body));

        // Overwrite should replace
        store(&conn, doc_id, "New body text.")?;
        let retrieved = get(&conn, doc_id)?;
        assert_eq!(retrieved.as_deref(), Some("New body text."));

        // Non-existent doc returns None
        let missing = get(&conn, doc_id + 999)?;
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_body_fts_search() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Physics Paper"))?;

        store(
            &conn,
            doc_id,
            "This paper explores quantum entanglement in superconducting circuits.",
        )?;

        let results = search_body(&conn, "quantum")?;
        assert!(
            results.contains(&doc_id),
            "body FTS search for 'quantum' should find doc {doc_id}, got {results:?}"
        );

        // Negative: a doc with unrelated body text should not match
        let other_id = db::documents::insert(&conn, &make_doc("Biology Paper"))?;
        store(
            &conn,
            other_id,
            "This paper discusses cell mitosis and DNA replication.",
        )?;
        let results = search_body(&conn, "quantum")?;
        assert!(
            !results.contains(&other_id),
            "body FTS search for 'quantum' should not find doc {other_id}"
        );

        Ok(())
    }

    #[test]
    fn test_body_fts_toggle() -> Result<()> {
        let conn = setup()?;

        // Doc whose metadata does NOT contain "quantum" but body does
        let doc_id = db::documents::insert(&conn, &make_doc("Untitled Research"))?;
        store(
            &conn,
            doc_id,
            "The phenomenon of quantum entanglement was observed in the experiment.",
        )?;

        // Toggle OFF: metadata-only search should NOT find the doc
        let metadata_only = db::search::search_documents_with_body(&conn, "quantum", false)?;
        assert!(
            !metadata_only.contains(&doc_id),
            "metadata-only search should exclude body-only matches, got {metadata_only:?}"
        );

        // Toggle ON: metadata + body search should find the doc
        let with_body = db::search::search_documents_with_body(&conn, "quantum", true)?;
        assert!(
            with_body.contains(&doc_id),
            "metadata+body search should include body-only matches, got {with_body:?}"
        );

        Ok(())
    }
}
