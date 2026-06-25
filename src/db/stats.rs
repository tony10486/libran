use anyhow::Result;
use rusqlite::Connection;

#[derive(Clone, Debug, Default)]
pub struct LibraryStats {
    pub total_documents: i64,
    pub documents_with_files: i64,
    pub documents_with_doi: i64,
    pub documents_with_arxiv: i64,
    pub reading_unread: i64,
    pub reading_reading: i64,
    pub reading_read: i64,
    pub rated_documents: i64,
    pub average_rating: f64,
    pub total_tags: i64,
    pub total_projects: i64,
    pub total_series: i64,
    pub total_authors: i64,
    pub total_citation_relations: i64,
    pub year_distribution: Vec<(i64, i64)>,
    pub top_authors: Vec<(String, i64)>,
    pub top_journals: Vec<(String, i64)>,
}

pub fn compute(conn: &Connection) -> Result<LibraryStats> {
    let mut stats = LibraryStats::default();

    stats.total_documents = conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
    stats.documents_with_files = conn.query_row("SELECT COUNT(*) FROM documents WHERE file_path IS NOT NULL AND trim(file_path) <> ''", [], |row| row.get(0))?;
    stats.documents_with_doi = conn.query_row("SELECT COUNT(*) FROM documents WHERE doi IS NOT NULL AND trim(doi) <> ''", [], |row| row.get(0))?;
    stats.documents_with_arxiv = conn.query_row("SELECT COUNT(*) FROM documents WHERE arxiv_id IS NOT NULL AND trim(arxiv_id) <> ''", [], |row| row.get(0))?;
    stats.reading_unread = conn.query_row("SELECT COUNT(*) FROM documents WHERE reading_status = 'unread'", [], |row| row.get(0))?;
    stats.reading_reading = conn.query_row("SELECT COUNT(*) FROM documents WHERE reading_status = 'reading'", [], |row| row.get(0))?;
    stats.reading_read = conn.query_row("SELECT COUNT(*) FROM documents WHERE reading_status = 'read'", [], |row| row.get(0))?;
    stats.rated_documents = conn.query_row("SELECT COUNT(*) FROM documents WHERE rating IS NOT NULL", [], |row| row.get(0))?;
    stats.average_rating = conn.query_row("SELECT COALESCE(AVG(rating), 0) FROM documents WHERE rating IS NOT NULL", [], |row| row.get(0))?;
    stats.total_tags = conn.query_row("SELECT COUNT(DISTINCT tag) FROM tags", [], |row| row.get(0))?;
    stats.total_projects = conn.query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;
    stats.total_series = conn.query_row("SELECT COUNT(*) FROM series", [], |row| row.get(0))?;
    stats.total_citation_relations = conn.query_row("SELECT COUNT(*) FROM citation_relations", [], |row| row.get(0))?;

    // Author count (distinct split authors)
    stats.total_authors = {
        let authors = crate::db::documents::list_authors(conn, 1)?;
        authors.len() as i64
    };

    // Year distribution
    let mut stmt = conn.prepare(
        "SELECT pub_year, COUNT(*) FROM documents WHERE pub_year IS NOT NULL GROUP BY pub_year ORDER BY pub_year",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    for row in rows {
        stats.year_distribution.push(row?);
    }

    // Top authors (max 10)
    stats.top_authors = crate::db::documents::list_authors(conn, 1)?
        .into_iter()
        .take(10)
        .collect();

    // Top journals
    let mut stmt = conn.prepare(
        "SELECT journal, COUNT(*) as cnt FROM documents WHERE journal IS NOT NULL AND trim(journal) <> '' GROUP BY journal ORDER BY cnt DESC LIMIT 10",
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    for row in rows {
        stats.top_journals.push(row?);
    }

    Ok(stats)
}
