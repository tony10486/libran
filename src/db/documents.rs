use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<i64>,
    pub title: String,
    pub authors: Option<String>,
    pub journal: Option<String>,
    pub pub_year: Option<i64>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub abstract_text: Option<String>,
    pub keywords: Option<String>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
    pub citation_key: Option<String>,
    pub source: Option<String>,
}

pub fn insert(conn: &Connection, doc: &Document) -> Result<i64> {
    conn.execute(
        "INSERT INTO documents (title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            doc.title,
            doc.authors,
            doc.journal,
            doc.pub_year,
            doc.doi,
            doc.arxiv_id,
            doc.abstract_text,
            doc.keywords,
            doc.file_path,
            doc.file_hash,
            doc.citation_key,
            doc.source.as_deref().unwrap_or("manual"),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source
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
        }))
    } else {
        Ok(None)
    }
}

pub fn find_by_doi(conn: &Connection, doi: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source
         FROM documents WHERE doi = ?1",
    )?;
    let mut rows = stmt.query(params![doi])?;
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
        }))
    } else {
        Ok(None)
    }
}

pub fn find_by_hash(conn: &Connection, hash: &str) -> Result<Option<Document>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source
         FROM documents WHERE file_hash = ?1",
    )?;
    let mut rows = stmt.query(params![hash])?;
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
        }))
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
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, pub_year, doi, arxiv_id, abstract, keywords, file_path, file_hash, citation_key, source
         FROM documents ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Document {
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
        })
    })?;
    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

pub fn update_citation_key(conn: &Connection, id: i64, key: &str) -> Result<()> {
    conn.execute(
        "UPDATE documents SET citation_key = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![key, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id])?;
    Ok(())
}
