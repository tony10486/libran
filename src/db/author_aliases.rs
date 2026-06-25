use anyhow::Result;
use rusqlite::{params, Connection};

#[derive(Clone, Debug)]
pub struct AuthorAlias {
    pub id: i64,
    pub alias_name: String,
    pub canonical_author_name: Option<String>,
    pub openalex_id: Option<String>,
    pub created_at: String,
}

pub fn list(conn: &Connection) -> Result<Vec<AuthorAlias>> {
    let mut stmt = conn.prepare(
        "SELECT id, alias_name, canonical_author_name, openalex_id, created_at
         FROM author_aliases ORDER BY alias_name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AuthorAlias {
            id: row.get(0)?,
            alias_name: row.get(1)?,
            canonical_author_name: row.get(2)?,
            openalex_id: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn insert(
    conn: &Connection,
    alias_name: &str,
    canonical_author_name: &str,
    openalex_id: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO author_aliases (alias_name, canonical_author_name, openalex_id)
         VALUES (?1, ?2, ?3)",
        params![alias_name, canonical_author_name, openalex_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM author_aliases WHERE id = ?1", params![id])?;
    Ok(())
}

/// Replace all occurrences of `source` in documents.authors with `canonical`.
/// Splits by ';', trims, replaces matching segments, rejoins.
pub fn merge_author_in_documents(
    conn: &Connection,
    source: &str,
    canonical: &str,
) -> Result<usize> {
    use crate::db::documents::split_authors;

    let mut stmt = conn.prepare("SELECT id, authors FROM documents WHERE authors IS NOT NULL AND trim(authors) <> ''")?;
    let docs: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let mut count = 0usize;
    for (id, authors) in docs {
        let parts = split_authors(&authors);
        let changed: Vec<String> = parts
            .iter()
            .map(|p| {
                if p.trim().eq_ignore_ascii_case(source.trim()) {
                    canonical.to_string()
                } else {
                    p.clone()
                }
            })
            .collect();
        let joined = changed.join("; ");
        if joined != authors {
            conn.execute(
                "UPDATE documents SET authors = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![joined, id],
            )?;
            count += 1;
        }
    }
    Ok(count)
}
