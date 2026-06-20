use anyhow::Result;
use rusqlite::Connection;

pub fn set_label(
    conn: &Connection,
    node_id: i64,
    lang: &str,
    label: &str,
    source: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO classification_labels (node_id, lang, label, source)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![node_id, lang, label, source],
    )?;
    Ok(())
}

pub fn get_label(conn: &Connection, node_id: i64, lang: &str) -> Result<Option<String>> {
    let result = conn
        .query_row(
            "SELECT label FROM classification_labels WHERE node_id = ?1 AND lang = ?2",
            rusqlite::params![node_id, lang],
            |row| row.get(0),
        )
        .ok();
    Ok(result)
}

pub fn resolve_label(
    conn: &Connection,
    node_id: i64,
    pref_label: &str,
    lang: &str,
) -> Result<String> {
    if let Some(translated) = get_label(conn, node_id, lang)? {
        return Ok(translated);
    }
    Ok(pref_label.to_string())
}
