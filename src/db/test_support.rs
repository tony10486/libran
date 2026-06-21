use anyhow::Result;
use rusqlite::Connection;

use crate::db;
use crate::db::documents::Document;

pub fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    db::init_database(&conn)?;
    Ok(conn)
}

pub fn make_doc(title: &str, authors: Option<&str>) -> Document {
    Document {
        id: None,
        title: title.to_string(),
        authors: authors.map(|s| s.to_string()),
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: None,
        source: None,
        rating: None,
    }
}

pub fn insert_doc(conn: &Connection, doc: &Document) -> Result<i64> {
    db::documents::insert(conn, doc)
}

pub fn set_db_version(conn: &Connection, version: i64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('db_version', ?1, CURRENT_TIMESTAMP)",
        rusqlite::params![version.to_string()],
    )?;
    Ok(())
}
