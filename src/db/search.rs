use anyhow::Result;
use rusqlite::{params, Connection};

use crate::db::fts_query::{build_search_plan, escape_like, normalize_nfc, SearchPlan};

pub fn search_documents(conn: &Connection, term: &str) -> Result<Vec<i64>> {
    let term = term.trim();
    if term.is_empty() {
        return Ok(Vec::new());
    }

    let term = normalize_nfc(term);

    match build_search_plan(&term) {
        SearchPlan::FtsMatch(escaped) => search_fts_match(conn, &escaped),
        SearchPlan::BigramMatch(escaped) => search_bigram_match(conn, &escaped),
        SearchPlan::ChoseongMatch(escaped) => search_choseong_match(conn, &escaped),
        SearchPlan::Like(t) => search_like(conn, &t),
    }
}

pub fn search_in_project(
    conn: &Connection,
    term: &str,
    project_id: i64,
) -> Result<Vec<i64>> {
    let term = term.trim();
    if term.is_empty() {
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

    let term = normalize_nfc(term);

    match build_search_plan(&term) {
        SearchPlan::FtsMatch(escaped) => search_in_project_fts(conn, &escaped, project_id),
        SearchPlan::BigramMatch(escaped) => {
            search_in_project_bigram(conn, &escaped, project_id)
        }
        SearchPlan::ChoseongMatch(escaped) => {
            search_in_project_choseong(conn, &escaped, project_id)
        }
        SearchPlan::Like(t) => search_in_project_like(conn, &t, project_id),
    }
}

fn search_fts_match(conn: &Connection, escaped: &str) -> Result<Vec<i64>> {
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

fn search_bigram_match(conn: &Connection, escaped: &str) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT rowid FROM documents_bigram_fts WHERE documents_bigram_fts MATCH ?1 ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_choseong_match(conn: &Connection, escaped: &str) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT rowid FROM documents_choseong_fts WHERE documents_choseong_fts MATCH ?1 ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_like(conn: &Connection, term: &str) -> Result<Vec<i64>> {
    let pattern = format!("%{}%", escape_like(term));
    let mut stmt = conn.prepare(
        "SELECT id FROM documents
         WHERE title LIKE ?1 ESCAPE '\\'
            OR authors LIKE ?1 ESCAPE '\\'
            OR journal LIKE ?1 ESCAPE '\\'
            OR abstract LIKE ?1 ESCAPE '\\'
            OR keywords LIKE ?1 ESCAPE '\\'",
    )?;
    let rows = stmt.query_map(params![pattern], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_in_project_fts(conn: &Connection, escaped: &str, project_id: i64) -> Result<Vec<i64>> {
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

fn search_in_project_bigram(
    conn: &Connection,
    escaped: &str,
    project_id: i64,
) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT d.id FROM documents d
         INNER JOIN project_documents pd ON d.id = pd.document_id
         INNER JOIN documents_bigram_fts fts ON d.id = fts.rowid
         WHERE pd.project_id = ?1 AND documents_bigram_fts MATCH ?2
         ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![project_id, escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_in_project_choseong(
    conn: &Connection,
    escaped: &str,
    project_id: i64,
) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT d.id FROM documents d
         INNER JOIN project_documents pd ON d.id = pd.document_id
         INNER JOIN documents_choseong_fts fts ON d.id = fts.rowid
         WHERE pd.project_id = ?1 AND documents_choseong_fts MATCH ?2
         ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![project_id, escaped], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn search_in_project_like(conn: &Connection, term: &str, project_id: i64) -> Result<Vec<i64>> {
    let pattern = format!("%{}%", escape_like(term));
    let mut stmt = conn.prepare(
        "SELECT d.id FROM documents d
         INNER JOIN project_documents pd ON d.id = pd.document_id
         WHERE pd.project_id = ?1
           AND (d.title LIKE ?2 ESCAPE '\\'
             OR d.authors LIKE ?2 ESCAPE '\\'
             OR d.journal LIKE ?2 ESCAPE '\\'
             OR d.abstract LIKE ?2 ESCAPE '\\'
             OR d.keywords LIKE ?2 ESCAPE '\\')",
    )?;
    let rows = stmt.query_map(params![project_id, pattern], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}
