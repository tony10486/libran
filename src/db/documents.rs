use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::db::fts_query::normalize_nfc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<i64>,
    pub title: String,
    pub authors: Option<String>,
    pub journal: Option<String>,
    pub conference: Option<String>,
    pub pub_year: Option<i64>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub abstract_text: Option<String>,
    pub keywords: Option<String>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
    pub citation_key: Option<String>,
    pub source: Option<String>,
    pub rating: Option<i64>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            id: None,
            title: String::new(),
            authors: None,
            journal: None,
            conference: None,
            pub_year: None,
            doi: None,
            arxiv_id: None,
            abstract_text: None,
            keywords: None,
            file_path: None,
            file_hash: None,
            citation_key: None,
            source: None,
            rating: None,
        }
    }
}

pub fn insert(conn: &Connection, doc: &Document) -> Result<i64> {
    conn.execute(
        "INSERT INTO documents (title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            normalize_nfc(&doc.title),
            doc.authors.as_deref().map(normalize_nfc),
            doc.journal.as_deref().map(normalize_nfc),
            doc.pub_year,
            doc.doi,
            doc.arxiv_id,
            doc.abstract_text.as_deref().map(normalize_nfc),
            doc.keywords.as_deref().map(normalize_nfc),
            doc.file_path,
            doc.file_hash,
            doc.citation_key,
            doc.source.as_deref().unwrap_or("manual"),
            doc.conference.as_deref().map(normalize_nfc),
            doc.rating,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating
         FROM documents WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Document {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            authors: row.get(2)?,
            journal: row.get(3)?,
            pub_year: row.get(4)?,
            doi: row.get(5)?,
            arxiv_id: row.get(6)?,
            abstract_text: row.get(7)?,
            keywords: row.get(8)?,
            file_path: row.get(9)?,
            file_hash: row.get(10)?,
            citation_key: row.get(11)?,
            source: row.get(12)?,
            conference: row.get(13)?,
            rating: row.get(14)?,
        }))
    } else {
        Ok(None)
    }
}

macro_rules! doc_from_row {
    ($row:expr) => {
        Document {
            id: Some($row.get(0)?),
            title: $row.get(1)?,
            authors: $row.get(2)?,
            journal: $row.get(3)?,
            pub_year: $row.get(4)?,
            doi: $row.get(5)?,
            arxiv_id: $row.get(6)?,
            abstract_text: $row.get(7)?,
            keywords: $row.get(8)?,
            file_path: $row.get(9)?,
            file_hash: $row.get(10)?,
            citation_key: $row.get(11)?,
            source: $row.get(12)?,
            conference: $row.get(13)?,
            rating: $row.get(14)?,
        }
    };
}

const DOCUMENT_COLS: &str =
    "id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating";

pub fn find_by_doi(conn: &Connection, doi: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        &format!("SELECT {} FROM documents WHERE doi = ?1", DOCUMENT_COLS),
    )?;
    let mut rows = stmt.query(params![doi])?;
    if let Some(row) = rows.next()? {
        Ok(Some(doc_from_row!(row)))
    } else {
        Ok(None)
    }
}

pub fn find_by_hash(conn: &Connection, hash: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        &format!("SELECT {} FROM documents WHERE file_hash = ?1", DOCUMENT_COLS),
    )?;
    let mut rows = stmt.query(params![hash])?;
    if let Some(row) = rows.next()? {
        Ok(Some(doc_from_row!(row)))
    } else {
        Ok(None)
    }
}

pub fn citation_key_exists(conn: &Connection, key: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE citation_key = ?1",
        params![key],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn list_all(conn: &Connection) -> Result<Vec<Document>> {
    let sql = format!("SELECT {} FROM documents ORDER BY id DESC", DOCUMENT_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| Ok(doc_from_row!(row)))?;
    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

pub fn update(conn: &Connection, doc: &Document) -> Result<()> {
    conn.execute(
        "UPDATE documents SET title = ?1, authors = ?2, journal = ?3, conference = ?4, pub_year = ?5,
         doi = ?6, arxiv_id = ?7, abstract = ?8, keywords = ?9,
         updated_at = CURRENT_TIMESTAMP
         WHERE id = ?10",
        params![
            normalize_nfc(&doc.title),
            doc.authors.as_deref().map(normalize_nfc),
            doc.journal.as_deref().map(normalize_nfc),
            doc.conference.as_deref().map(normalize_nfc),
            doc.pub_year,
            doc.doi,
            doc.arxiv_id,
            doc.abstract_text.as_deref().map(normalize_nfc),
            doc.keywords.as_deref().map(normalize_nfc),
            doc.id,
        ],
    )?;
    Ok(())
}

pub fn update_citation_key(conn: &Connection, id: i64, key: &str) -> Result<()> {
    conn.execute(
        "UPDATE documents SET citation_key = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![key, id],
    )?;
    Ok(())
}

pub fn update_rating(conn: &Connection, id: i64, rating: Option<i64>) -> Result<()> {
    conn.execute(
        "UPDATE documents SET rating = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![rating, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Tag helpers ──

pub fn get_tags(conn: &Connection, document_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM tags WHERE document_id = ?1 ORDER BY tag")?;
    let rows = stmt.query_map(params![document_id], |row| row.get::<_, String>(0))?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn add_tag(conn: &Connection, document_id: i64, tag: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (document_id, tag) VALUES (?1, ?2)",
        params![document_id, tag],
    )?;
    Ok(())
}

pub fn remove_tag(conn: &Connection, document_id: i64, tag: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM tags WHERE document_id = ?1 AND tag = ?2",
        params![document_id, tag],
    )?;
    Ok(())
}

// ── Citation helpers ──

/// Add a citation relation with match metadata.
pub fn add_citation_with_status(
    conn: &Connection,
    citing_id: i64,
    cited_id: i64,
    match_status: &str,
    confidence: f64,
    raw_ref_text: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO citation_relations (citing_id, cited_id, match_status, confidence, raw_ref_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
        params![citing_id, cited_id, match_status, confidence, raw_ref_text],
    )?;
    Ok(())
}

/// Add a citation relation: `citing_id` cites `cited_id` (legacy, defaults to manual).
pub fn add_citation(conn: &Connection, citing_id: i64, cited_id: i64) -> Result<()> {
    add_citation_with_status(conn, citing_id, cited_id, "manual", 1.0, None)
}

/// Remove a citation relation.
pub fn remove_citation(conn: &Connection, citing_id: i64, cited_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM citation_relations WHERE citing_id = ?1 AND cited_id = ?2",
        params![citing_id, cited_id],
    )?;
    Ok(())
}

/// Get all document IDs that this document cites.
pub fn get_cited_docs(conn: &Connection, document_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT cited_id FROM citation_relations WHERE citing_id = ?1",
    )?;
    let rows = stmt.query_map(params![document_id], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

/// Get all document IDs that cite this document.
pub fn get_citing_docs(conn: &Connection, document_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT citing_id FROM citation_relations WHERE cited_id = ?1",
    )?;
    let rows = stmt.query_map(params![document_id], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

/// Store a reference-section extraction attempt.
pub fn save_reference_extraction(
    conn: &Connection,
    doc_id: i64,
    section_text: &str,
    extraction_method: &str,
    success: i32,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO reference_extractions (doc_id, section_text, extraction_method, extraction_success, extracted_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![doc_id, section_text, extraction_method, success],
    )?;
    Ok(())
}

/// Check if reference extraction was already attempted for a document.
pub fn has_reference_extraction(conn: &Connection, doc_id: i64) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reference_extractions WHERE doc_id = ?1",
        params![doc_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Aggregate (author, doc_count) pairs with at least `min_count` docs.
/// Splits `documents.authors` by ';', trims, NFC-normalizes, and sorts by
/// count desc then name asc.
pub fn list_authors(conn: &Connection, min_count: usize) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare("SELECT authors FROM documents WHERE authors IS NOT NULL AND trim(authors) <> ''")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for row in rows {
        let authors = row?;
        for name in split_authors(&authors) {
            *counts.entry(normalize_nfc(&name).to_string()).or_insert(0) += 1;
        }
    }
    drop(stmt);

    let mut out: Vec<(String, i64)> = counts
        .into_iter()
        .filter(|(_, c)| (*c as usize) >= min_count)
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(out)
}

pub fn split_authors(authors: &str) -> Vec<String> {
    let mut result = Vec::new();
    for seg in authors.split(';') {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        let lower = seg.to_lowercase();
        if let Some(pos) = lower.find(" and ") {
            let before = seg[..pos].trim();
            let after = seg[pos + 5..].trim();
            if !before.is_empty() {
                result.push(before.to_string());
            }
            if !after.is_empty() {
                result.push(after.to_string());
            }
        } else {
            result.push(seg.to_string());
        }
    }
    result
}
