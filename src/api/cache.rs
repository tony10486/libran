use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};

pub fn get_cached(conn: &Connection, cache_key: &str) -> Result<Option<String>> {
    let now = Utc::now();
    let result = conn.query_row(
        "SELECT response_json FROM api_cache WHERE cache_key = ?1 AND expires_at > ?2",
        params![cache_key, now.to_rfc3339()],
        |row| row.get(0),
    );
    match result {
        Ok(json) => Ok(Some(json)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_cached(conn: &Connection, cache_key: &str, source: &str, json: &str) -> Result<()> {
    let now = Utc::now();
    let expires = now + Duration::days(30);
    conn.execute(
        "INSERT OR REPLACE INTO api_cache (cache_key, source, response_json, fetched_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![cache_key, source, json, now.to_rfc3339(), expires.to_rfc3339()],
    )?;
    Ok(())
}

pub fn cleanup_expired(conn: &Connection) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "DELETE FROM api_cache WHERE expires_at <= ?1",
        params![now.to_rfc3339()],
    )?;
    Ok(())
}
