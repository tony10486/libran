use anyhow::Result;
use rusqlite::Connection;

pub struct CustomField {
    pub id: i64,
    pub key: String,
    pub value: String,
}

pub fn add_field(conn: &Connection, doc_id: i64, key: &str, value: &str) -> Result<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO document_custom_fields (document_id, field_key, field_value, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params![doc_id, key, value],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_fields(conn: &Connection, doc_id: i64) -> Result<Vec<CustomField>> {
    let mut stmt = conn.prepare(
        "SELECT id, field_key, COALESCE(field_value, '') FROM document_custom_fields
         WHERE document_id = ?1 ORDER BY field_key",
    )?;
    let rows = stmt.query_map(rusqlite::params![doc_id], |row| {
        Ok(CustomField {
            id: row.get(0)?,
            key: row.get(1)?,
            value: row.get(2)?,
        })
    })?;
    let mut fields = Vec::new();
    for row in rows {
        fields.push(row?);
    }
    Ok(fields)
}

pub fn delete_field(conn: &Connection, doc_id: i64, field_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM document_custom_fields WHERE id = ?1 AND document_id = ?2",
        rusqlite::params![field_id, doc_id],
    )?;
    Ok(())
}

pub fn update_field(conn: &Connection, doc_id: i64, field_id: i64, value: &str) -> Result<()> {
    conn.execute(
        "UPDATE document_custom_fields SET field_value = ?1, updated_at = datetime('now')
         WHERE id = ?2 AND document_id = ?3",
        rusqlite::params![value, field_id, doc_id],
    )?;
    Ok(())
}
