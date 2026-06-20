use anyhow::Result;
use rusqlite::{params, Connection};

pub fn search_documents(conn: &Connection, term: &str) -> Result<Vec<i64>> {
    if term.trim().is_empty() {
        return Ok(Vec::new());
    }
    let escaped = escape_fts_query(term);
    let mut stmt = conn.prepare(
        "SELECT rowid FROM documents_fts WHERE documents_fts MATCH ?1 ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

pub fn search_in_project(
    conn: &Connection,
    term: &str,
    project_id: i64,
) -> Result<Vec<i64>> {
    if term.trim().is_empty() {
        let mut stmt = conn.prepare(
            "SELECT document_id FROM project_documents WHERE project_id = ?1 ORDER BY added_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| row.get::<_, i64>(0))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }
        return Ok(ids);
    }
    let escaped = escape_fts_query(term);
    let mut stmt = conn.prepare(
        "SELECT d.id FROM documents d
         INNER JOIN project_documents pd ON d.id = pd.document_id
         INNER JOIN documents_fts fts ON d.id = fts.rowid
         WHERE pd.project_id = ?1 AND documents_fts MATCH ?2
         ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![project_id, escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn escape_fts_query(term: &str) -> String {
    if term.len() < 3 {
        return format!("\"{}\"", term);
    }
    format!("\"{}\"", term.replace('"', "\"\""))
}
