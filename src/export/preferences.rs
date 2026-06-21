use anyhow::Result;
use rusqlite::Connection;

use crate::citation::text::styles::{CitationLanguage, CitationStyle};
use crate::export::ExportFormat;

const KEY_FORMAT: &str = "export_last_format";
const KEY_STYLE: &str = "export_last_style";
const KEY_LANGUAGE: &str = "export_last_language";

pub fn save(
    conn: &Connection,
    format: ExportFormat,
    style: CitationStyle,
    language: CitationLanguage,
) -> Result<()> {
    save_value(conn, KEY_FORMAT, format.as_str())?;
    save_value(conn, KEY_STYLE, style.as_str())?;
    save_value(conn, KEY_LANGUAGE, language.as_str())?;
    Ok(())
}

pub fn load(conn: &Connection) -> Option<(ExportFormat, CitationStyle, CitationLanguage)> {
    let format = load_value(conn, KEY_FORMAT).and_then(|s| ExportFormat::from_str(&s))?;
    let style = load_value(conn, KEY_STYLE).and_then(|s| CitationStyle::from_str(&s))?;
    let language = load_value(conn, KEY_LANGUAGE).and_then(|s| CitationLanguage::from_str(&s))?;
    Some((format, style, language))
}

fn save_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

fn load_value(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .expect("create app_config table");
        conn
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let conn = in_memory_conn();
        save(
            &conn,
            ExportFormat::Ris,
            CitationStyle::Nature,
            CitationLanguage::Korean,
        )
        .expect("save preferences");
        let loaded = load(&conn);
        assert_eq!(
            loaded,
            Some((
                ExportFormat::Ris,
                CitationStyle::Nature,
                CitationLanguage::Korean
            ))
        );
    }

    #[test]
    fn test_load_returns_none_when_empty() {
        let conn = in_memory_conn();
        assert!(load(&conn).is_none());
    }

    #[test]
    fn test_save_overwrites_previous_value() {
        let conn = in_memory_conn();
        save(
            &conn,
            ExportFormat::Bibtex,
            CitationStyle::Apa7th,
            CitationLanguage::English,
        )
        .expect("save first");
        save(
            &conn,
            ExportFormat::Csv,
            CitationStyle::Mla9thInText,
            CitationLanguage::Japanese,
        )
        .expect("save second");
        let loaded = load(&conn);
        assert_eq!(
            loaded,
            Some((
                ExportFormat::Csv,
                CitationStyle::Mla9thInText,
                CitationLanguage::Japanese
            ))
        );
    }
}
