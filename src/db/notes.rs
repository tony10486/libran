use anyhow::Result;
use rusqlite::{Connection, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub id: Option<i64>,
    pub document_id: i64,
    pub content: String,
    pub note_type: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub fn list(conn: &Connection, document_id: i64) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, content, note_type, created_at, updated_at
         FROM document_notes
         WHERE document_id = ?1
         ORDER BY updated_at DESC, id DESC",
    )?;
    let notes = stmt
        .query_map(params![document_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                document_id: row.get(1)?,
                content: row.get(2)?,
                note_type: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(notes)
}

pub fn get_by_id(conn: &Connection, note_id: i64) -> Result<Option<Note>> {
    let result = conn.query_row(
        "SELECT id, document_id, content, note_type, created_at, updated_at
         FROM document_notes
         WHERE id = ?1",
        params![note_id],
        |row| {
            Ok(Note {
                id: row.get(0)?,
                document_id: row.get(1)?,
                content: row.get(2)?,
                note_type: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    );
    match result {
        Ok(note) => Ok(Some(note)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn create(conn: &Connection, document_id: i64, content: &str, note_type: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO document_notes (document_id, content, note_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        params![document_id, content, note_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, note_id: i64, content: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_notes SET content = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![note_id, content],
    )?;
    Ok(())
}

pub fn delete_by_id(conn: &Connection, note_id: i64) -> Result<()> {
    conn.execute("DELETE FROM document_notes WHERE id = ?1", params![note_id])?;
    Ok(())
}
