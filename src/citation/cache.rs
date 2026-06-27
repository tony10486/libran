use anyhow::Result;
use rusqlite::Connection;

use super::graph::RenderMode;

#[derive(Clone, Debug)]
pub struct GraphCacheEntry {
    pub cache_key: String,
    pub graph_data: String,
    pub edge_version: i64,
    pub doc_count: i64,
    pub render_mode: String,
}

pub fn build_cache_key(doc_ids: &[i64]) -> String {
    let mut sorted: Vec<i64> = doc_ids.to_vec();
    sorted.sort();
    sorted
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(":")
}

pub fn compute_edge_version(conn: &Connection, doc_ids: &[i64]) -> Result<i64> {
    if doc_ids.is_empty() {
        return Ok(0);
    }
    let placeholders: Vec<String> = doc_ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT COALESCE(SUM(id), 0) FROM citation_relations WHERE citing_id IN ({}) OR cited_id IN ({})",
        placeholders.join(","),
        placeholders.join(",")
    );
    let params: Vec<&dyn rusqlite::types::ToSql> = doc_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .chain(doc_ids.iter().map(|id| id as &dyn rusqlite::types::ToSql))
        .collect();
    let version: i64 = conn.query_row(&sql, params.as_slice(), |row| row.get(0))?;
    Ok(version)
}

const REGEN_THRESHOLD: i64 = 5;

pub fn should_regenerate(conn: &Connection, cache_key: &str, new_doc_ids: &[i64]) -> Result<bool> {
    let cached = lookup_cache(conn, cache_key)?;

    let Some(entry) = cached else {
        return Ok(true);
    };

    let current_edge_version = compute_edge_version(conn, new_doc_ids)?;
    if current_edge_version != entry.edge_version {
        return Ok(true);
    }

    let new_docs_in_set: std::collections::HashSet<i64> = new_doc_ids.iter().copied().collect();
    let cached_docs: std::collections::HashSet<i64> = cache_key
        .split(':')
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();

    let delta_count = new_docs_in_set.difference(&cached_docs).count() as i64;

    Ok(delta_count >= REGEN_THRESHOLD)
}

pub fn lookup_cache(conn: &Connection, cache_key: &str) -> Result<Option<GraphCacheEntry>> {
    let result = conn.query_row(
        "SELECT cache_key, graph_data, edge_version, doc_count, render_mode FROM graph_cache WHERE cache_key = ?1",
        rusqlite::params![cache_key],
        |row| {
            Ok(GraphCacheEntry {
                cache_key: row.get(0)?,
                graph_data: row.get(1)?,
                edge_version: row.get(2)?,
                doc_count: row.get(3)?,
                render_mode: row.get(4)?,
            })
        },
    );

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn store_cache(
    conn: &Connection,
    cache_key: &str,
    graph_data: &str,
    edge_version: i64,
    doc_count: i64,
    render_mode: &RenderMode,
) -> Result<()> {
    let mode_str = match render_mode {
        RenderMode::Visual => "visual",
        RenderMode::Table => "table",
    };
    conn.execute(
        "INSERT OR REPLACE INTO graph_cache (cache_key, graph_data, edge_version, doc_count, render_mode, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![cache_key, graph_data, edge_version, doc_count, mode_str],
    )?;
    Ok(())
}

pub fn invalidate_cache(conn: &Connection, cache_key: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM graph_cache WHERE cache_key = ?1",
        rusqlite::params![cache_key],
    )?;
    Ok(())
}

pub fn invalidate_all_cache(conn: &Connection) -> Result<usize> {
    let count = conn.execute("DELETE FROM graph_cache", [])?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cache_key_sorted() {
        assert_eq!(build_cache_key(&[3, 1, 2]), "1:2:3");
    }

    #[test]
    fn test_build_cache_key_single() {
        assert_eq!(build_cache_key(&[42]), "42");
    }

    #[test]
    fn test_build_cache_key_empty() {
        assert_eq!(build_cache_key(&[]), "");
    }
}
