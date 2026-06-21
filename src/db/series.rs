use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A series bundles multiple issues/volumes of the same publication
/// (e.g., "Lecture Notes in Mathematics" vol 1/2/3, or all issues of a journal).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Series {
    pub id: Option<i64>,
    pub name: String,
    pub publisher: Option<String>,
    pub issn: Option<String>,
}

/// A document's membership in a series, with per-issue metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentSeriesEntry {
    pub document_id: i64,
    pub series_id: i64,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub sort_order: i64,
}

/// A proposed series discovered by auto-grouping (e.g., by journal name).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeriesProposal {
    pub name: String,
    pub publisher: Option<String>,
    pub document_ids: Vec<i64>,
}

// ── CRUD ──

pub fn create_series(conn: &Connection, name: &str, publisher: Option<&str>, issn: Option<&str>) -> Result<i64> {
    conn.execute(
        "INSERT INTO series (name, publisher, issn) VALUES (?1, ?2, ?3)",
        params![name, publisher, issn],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_series(conn: &Connection) -> Result<Vec<Series>> {
    let mut stmt = conn.prepare("SELECT id, name, publisher, issn FROM series ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Series {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            publisher: row.get(2)?,
            issn: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Series>> {
    let mut stmt = conn.prepare("SELECT id, name, publisher, issn FROM series WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Series {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            publisher: row.get(2)?,
            issn: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_series(conn: &Connection, series_id: i64) -> Result<()> {
    conn.execute("DELETE FROM series WHERE id = ?1", params![series_id])?;
    Ok(())
}

// ── Document-Series membership ──

pub fn add_document(
    conn: &Connection,
    series_id: i64,
    document_id: i64,
    volume: Option<&str>,
    issue: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO document_series (document_id, series_id, volume, issue)
         VALUES (?1, ?2, ?3, ?4)",
        params![document_id, series_id, volume, issue],
    )?;
    Ok(())
}

pub fn remove_document(conn: &Connection, series_id: i64, document_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM document_series WHERE series_id = ?1 AND document_id = ?2",
        params![series_id, document_id],
    )?;
    Ok(())
}

/// List document IDs in a series, ordered by sort_order then volume then issue.
pub fn list_documents(conn: &Connection, series_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT document_id FROM document_series
         WHERE series_id = ?1
         ORDER BY sort_order ASC, volume ASC, issue ASC, added_at DESC",
    )?;
    let rows = stmt.query_map(params![series_id], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

/// Count documents in a series.
pub fn count_documents(conn: &Connection, series_id: i64) -> Result<i64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM document_series WHERE series_id = ?1",
        params![series_id],
        |row| row.get(0),
    )?;
    Ok(n)
}

/// List all series a document belongs to.
pub fn list_series_for_document(conn: &Connection, document_id: i64) -> Result<Vec<Series>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.publisher, s.issn
         FROM series s
         INNER JOIN document_series ds ON ds.series_id = s.id
         WHERE ds.document_id = ?1
         ORDER BY s.name",
    )?;
    let rows = stmt.query_map(params![document_id], |row| {
        Ok(Series {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            publisher: row.get(2)?,
            issn: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

// ── Auto-grouping ──

/// Scan all documents and propose series based on shared journal name.
/// Returns one proposal per journal that has 2+ documents and no existing
/// series of the same name. Documents already in a series of that name are
/// excluded from the proposal.
pub fn propose_series_by_journal(conn: &Connection) -> Result<Vec<SeriesProposal>> {
    // Collect (journal, document_id, publisher_hint) for all docs with a journal.
    // Publisher is unknown in the current schema, so we leave it None; the journal
    // name itself serves as the series name.
    let mut stmt = conn.prepare(
        "SELECT id, journal FROM documents
         WHERE journal IS NOT NULL AND trim(journal) <> ''
         ORDER BY journal, id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    // Group document ids by journal name (NFC-normalized at insert time).
    let mut groups: std::collections::HashMap<String, Vec<i64>> = std::collections::HashMap::new();
    for row in rows {
        let (id, journal) = row?;
        groups.entry(journal).or_default().push(id);
    }
    drop(stmt);

    // Filter: journals with 2+ docs and no existing series of that name.
    let mut proposals: Vec<SeriesProposal> = Vec::new();
    for (name, doc_ids) in groups {
        if doc_ids.len() < 2 {
            continue;
        }
        if let Ok(Some(_)) = get_by_name(conn, &name) {
            // Series already exists; skip proposing a duplicate.
            continue;
        }
        proposals.push(SeriesProposal {
            name,
            publisher: None,
            document_ids: doc_ids,
        });
    }
    // Stable ordering by doc count descending (most valuable bundling first), then name.
    proposals.sort_by(|a, b| {
        b.document_ids.len().cmp(&a.document_ids.len()).then(a.name.cmp(&b.name))
    });
    Ok(proposals)
}

/// Create series from proposals and assign their documents. Skips proposals
/// whose name already matches an existing series (idempotent). Returns the
/// series IDs that were created or reused.
pub fn apply_proposals(conn: &Connection, proposals: &[SeriesProposal]) -> Result<Vec<i64>> {
    let mut created = Vec::new();
    for p in proposals {
        let series_id = if let Ok(Some(existing)) = get_by_name(conn, &p.name) {
            existing.id.unwrap_or(0)
        } else {
            create_series(conn, &p.name, p.publisher.as_deref(), None)?
        };
        for doc_id in &p.document_ids {
            add_document(conn, series_id, *doc_id, None, None)?;
        }
        created.push(series_id);
    }
    Ok(created)
}

/// One-shot bundling by journal: creates new series for journals without one,
/// and backfills existing series with any non-member documents. Idempotent —
/// running twice with no new documents returns an empty Vec. Returns series IDs
/// that were created, reused, or touched.
pub fn auto_group_by_journal(conn: &Connection) -> Result<Vec<i64>> {
    use std::collections::HashSet;

    let proposals = propose_series_by_journal(conn)?;
    let mut ids = apply_proposals(conn, &proposals)?;

    let mut stmt = conn.prepare(
        "SELECT id, journal FROM documents
         WHERE journal IS NOT NULL AND trim(journal) <> ''
         ORDER BY journal, id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut groups: std::collections::HashMap<String, Vec<i64>> = std::collections::HashMap::new();
    for row in rows {
        let (id, journal) = row?;
        groups.entry(journal).or_default().push(id);
    }
    drop(stmt);

    for (name, doc_ids) in groups {
        let Some(series) = get_by_name(conn, &name)? else {
            continue;
        };
        let series_id = series.id.unwrap_or(0);

        let members: HashSet<i64> = list_documents(conn, series_id)?.into_iter().collect();

        let mut touched = false;
        for doc_id in doc_ids {
            if !members.contains(&doc_id) {
                add_document(conn, series_id, doc_id, None, None)?;
                touched = true;
            }
        }
        if touched && !ids.contains(&series_id) {
            ids.push(series_id);
        }
    }

    Ok(ids)
}
