use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;

use crate::citation::match_refs::normalize_title;
use crate::db::fts_query::normalize_nfc;

const DUP_THRESHOLD: f64 = 0.75;
const TITLE_WEIGHT: f64 = 3.0;
const AUTHOR_WEIGHT: f64 = 2.5;
const YEAR_WEIGHT: f64 = 1.0;

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
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub page_start: Option<String>,
    pub page_end: Option<String>,
    pub publisher: Option<String>,
    pub city: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub issn: Option<String>,
    pub url: Option<String>,
    pub accessed_date: Option<String>,
    pub reading_status: Option<String>,
    pub reading_progress: Option<i64>,
    pub queue_position: Option<i64>,
    pub item_type: String,
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
            volume: None,
            issue: None,
            page_start: None,
            page_end: None,
            publisher: None,
            city: None,
            edition: None,
            isbn: None,
            issn: None,
            url: None,
            accessed_date: None,
            reading_status: None,
            reading_progress: None,
            queue_position: None,
            item_type: "misc".to_string(),
        }
    }
}

pub fn insert(conn: &Connection, doc: &Document) -> Result<i64> {
    conn.execute(
        "INSERT INTO documents (title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date, item_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
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
            doc.volume,
            doc.issue,
            doc.page_start,
            doc.page_end,
            doc.publisher,
            doc.city,
            doc.edition,
            doc.isbn,
            doc.issn,
            doc.url,
            doc.accessed_date,
            doc.item_type,
        ],
    )?;
    let doc_id = conn.last_insert_rowid();
    crate::db::creators::sync_from_authors(conn, doc_id, doc.authors.as_deref())?;
    Ok(doc_id)
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date, reading_status, reading_progress, queue_position, item_type
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
            volume: row.get(15)?,
            issue: row.get(16)?,
            page_start: row.get(17)?,
            page_end: row.get(18)?,
            publisher: row.get(19)?,
            city: row.get(20)?,
            edition: row.get(21)?,
            isbn: row.get(22)?,
            issn: row.get(23)?,
            url: row.get(24)?,
            accessed_date: row.get(25)?,
            reading_status: row.get(26)?,
            reading_progress: row.get(27)?,
            queue_position: row.get(28)?,
            item_type: row.get(29)?,
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
            volume: $row.get(15)?,
            issue: $row.get(16)?,
            page_start: $row.get(17)?,
            page_end: $row.get(18)?,
            publisher: $row.get(19)?,
            city: $row.get(20)?,
            edition: $row.get(21)?,
            isbn: $row.get(22)?,
            issn: $row.get(23)?,
            url: $row.get(24)?,
            accessed_date: $row.get(25)?,
            reading_status: $row.get(26)?,
            reading_progress: $row.get(27)?,
            queue_position: $row.get(28)?,
            item_type: $row.get(29)?,
        }
    };
}

const DOCUMENT_COLS: &str = "id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source, conference, rating, volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date, reading_status, reading_progress, queue_position, item_type";

pub fn find_by_doi(conn: &Connection, doi: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM documents WHERE doi = ?1",
        DOCUMENT_COLS
    ))?;
    let mut rows = stmt.query(params![doi])?;
    if let Some(row) = rows.next()? {
        Ok(Some(doc_from_row!(row)))
    } else {
        Ok(None)
    }
}

pub fn find_by_hash(conn: &Connection, hash: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM documents WHERE file_hash = ?1",
        DOCUMENT_COLS
    ))?;
    let mut rows = stmt.query(params![hash])?;
    if let Some(row) = rows.next()? {
        Ok(Some(doc_from_row!(row)))
    } else {
        Ok(None)
    }
}

/// Find fuzzy duplicate documents using weighted title/author/year similarity.
/// Returns `(doc_id, score)` pairs with score >= 0.75, sorted by score descending.
/// Weights: title 3.0, author 2.5, year 1.0 (exact match). Fields that are None
/// in either the query or candidate are skipped (weight excluded from normalization).
pub fn find_duplicates(conn: &Connection, query: &Document) -> Result<Vec<(i64, f64)>> {
    let mut stmt = conn.prepare("SELECT id, title, authors, pub_year FROM documents")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let authors: Option<String> = row.get(2)?;
        let pub_year: Option<i64> = row.get(3)?;
        Ok((id, title, authors, pub_year))
    })?;

    let norm_query_title = normalize_title(&query.title);
    let norm_query_authors = query.authors.as_deref().map(|a| a.to_lowercase());
    let query_year = query.pub_year;

    let mut matches: Vec<(i64, f64)> = Vec::new();
    for row in rows {
        let (id, title, authors, pub_year) = match row {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut weight_sum = 0.0;
        let mut score_sum = 0.0;

        let title_score = jaro_winkler(&norm_query_title, &normalize_title(&title));
        score_sum += TITLE_WEIGHT * title_score;
        weight_sum += TITLE_WEIGHT;

        if let (Some(qa), Some(ca)) = (norm_query_authors.as_deref(), authors.as_deref()) {
            let author_score = jaro_winkler(qa, &ca.to_lowercase());
            score_sum += AUTHOR_WEIGHT * author_score;
            weight_sum += AUTHOR_WEIGHT;
        }

        if let (Some(qy), Some(cy)) = (query_year, pub_year) {
            let year_score = if qy == cy { 1.0 } else { 0.0 };
            score_sum += YEAR_WEIGHT * year_score;
            weight_sum += YEAR_WEIGHT;
        }

        let score = score_sum / weight_sum;
        if score >= DUP_THRESHOLD {
            matches.push((id, score));
        }
    }

    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(matches)
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
         volume = ?10, issue = ?11, page_start = ?12, page_end = ?13, publisher = ?14, city = ?15,
         edition = ?16, isbn = ?17, issn = ?18, url = ?19, accessed_date = ?20,
         item_type = ?21,
         updated_at = CURRENT_TIMESTAMP
         WHERE id = ?22",
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
            doc.volume,
            doc.issue,
            doc.page_start,
            doc.page_end,
            doc.publisher,
            doc.city,
            doc.edition,
            doc.isbn,
            doc.issn,
            doc.url,
            doc.accessed_date,
            doc.item_type,
            doc.id,
        ],
    )?;
    if let Some(id) = doc.id {
        crate::db::creators::sync_from_authors(conn, id, doc.authors.as_deref())?;
    }
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

pub fn update_reading_status(conn: &Connection, id: i64, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE documents SET reading_status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

pub fn update_reading_progress(conn: &Connection, id: i64, progress: i64) -> Result<()> {
    conn.execute(
        "UPDATE documents SET reading_progress = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![progress, id],
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

pub fn set_tag_color(conn: &Connection, tag: &str, color: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE tags SET color = ?1 WHERE tag = ?2",
        params![color, tag],
    )?;
    Ok(())
}

pub fn get_tags_with_color(conn: &Connection) -> Result<Vec<(String, Option<String>)>> {
    let mut stmt = conn.prepare("SELECT tag, color FROM tags GROUP BY tag ORDER BY tag")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row?);
    }
    Ok(tags)
}

pub fn list_favorites(conn: &Connection) -> Result<Vec<Document>> {
    let sql = format!(
        "SELECT {} FROM documents WHERE rating = 5 ORDER BY id DESC",
        DOCUMENT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| Ok(doc_from_row!(row)))?;
    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

// ── Reading queue / TBR ──

pub fn add_to_queue(conn: &Connection, doc_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE documents SET queue_position = COALESCE(
            (SELECT MAX(queue_position) FROM documents WHERE queue_position IS NOT NULL), 0
         ) + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![doc_id],
    )?;
    Ok(())
}

pub fn remove_from_queue(conn: &Connection, doc_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE documents SET queue_position = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![doc_id],
    )?;
    conn.execute_batch(
        "WITH renumbered AS (
            SELECT id, ROW_NUMBER() OVER (ORDER BY queue_position) AS new_pos
            FROM documents
            WHERE queue_position IS NOT NULL
        )
        UPDATE documents
        SET queue_position = (SELECT new_pos FROM renumbered WHERE renumbered.id = documents.id)",
    )?;
    Ok(())
}

pub fn get_queue(conn: &Connection) -> Result<Vec<Document>> {
    let sql = format!(
        "SELECT {} FROM documents WHERE queue_position IS NOT NULL ORDER BY queue_position",
        DOCUMENT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| Ok(doc_from_row!(row)))?;
    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

pub fn reorder_queue(conn: &Connection, doc_id: i64, new_position: usize) -> Result<()> {
    let current: Option<i64> = conn
        .query_row(
            "SELECT queue_position FROM documents WHERE id = ?1",
            params![doc_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let current = match current {
        Some(pos) => pos,
        None => return Ok(()),
    };

    let new_pos = new_position as i64 + 1;

    if new_pos < current {
        conn.execute(
            "UPDATE documents SET queue_position = queue_position + 1
             WHERE queue_position IS NOT NULL AND queue_position >= ?1 AND queue_position < ?2",
            params![new_pos, current],
        )?;
    } else if new_pos > current {
        conn.execute(
            "UPDATE documents SET queue_position = queue_position - 1
             WHERE queue_position IS NOT NULL AND queue_position > ?1 AND queue_position <= ?2",
            params![current, new_pos],
        )?;
    } else {
        return Ok(());
    }

    conn.execute(
        "UPDATE documents SET queue_position = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![new_pos, doc_id],
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
    let mut stmt = conn.prepare("SELECT cited_id FROM citation_relations WHERE citing_id = ?1")?;
    let rows = stmt.query_map(params![document_id], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

/// Get all document IDs that cite this document.
pub fn get_citing_docs(conn: &Connection, document_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT citing_id FROM citation_relations WHERE cited_id = ?1")?;
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
    let mut stmt = conn.prepare(
        "SELECT authors FROM documents WHERE authors IS NOT NULL AND trim(authors) <> ''",
    )?;
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
