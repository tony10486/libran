use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: i64,
    pub name: String,
    pub fts_query: Option<String>,
    pub filters_json: Option<String>,
    pub created_at: String,
}

pub fn list(conn: &Connection) -> Result<Vec<SavedSearch>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fts_query, filters_json, created_at FROM saved_searches ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SavedSearch {
            id: row.get(0)?,
            name: row.get(1)?,
            fts_query: row.get(2)?,
            filters_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<SavedSearch>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fts_query, filters_json, created_at FROM saved_searches WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(SavedSearch {
            id: row.get(0)?,
            name: row.get(1)?,
            fts_query: row.get(2)?,
            filters_json: row.get(3)?,
            created_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn insert(conn: &Connection, name: &str, fts_query: Option<&str>, filters_json: Option<&str>) -> Result<i64> {
    conn.execute(
        "INSERT INTO saved_searches (name, fts_query, filters_json) VALUES (?1, ?2, ?3)",
        params![name, fts_query, filters_json],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
    Ok(())
}
