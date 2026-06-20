use anyhow::Result;
use rusqlite::{params, Connection};

pub fn get(conn: &Connection, document_id: i64) -> Result<Option<String>> {
    let result: std::result::Result<String, rusqlite::Error> = conn.query_row(
        "SELECT content FROM document_notes WHERE document_id = ?1",
        params![document_id],
        |row| row.get(0),
    );
    match result {
        Ok(content) => Ok(Some(content)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set(conn: &Connection, document_id: i64, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO document_notes (document_id, content, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(document_id) DO UPDATE SET content = excluded.content, updated_at = datetime('now')",
        params![document_id, content],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, document_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM document_notes WHERE document_id = ?1",
        params![document_id],
    )?;
    Ok(())
}
