use anyhow::Result;
use rusqlite::Connection;

/// Migration versions for tracking applied migrations.
const MIGRATION_KEY: &str = "db_version";

fn get_version(conn: &Connection) -> i64 {
    let result: std::result::Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        rusqlite::params![MIGRATION_KEY],
        |row| row.get(0),
    );
    match result {
        Ok(v) => v.parse::<i64>().unwrap_or(0),
        Err(_) => 0,
    }
}

fn set_version(conn: &Connection, version: i64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        rusqlite::params![MIGRATION_KEY, version.to_string()],
    )?;
    Ok(())
}

pub fn run(conn: &Connection) -> Result<()> {
    let version = get_version(conn);

    if version < 1 {
        // Migration 1: add conference column to documents
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN conference TEXT", []);
        set_version(conn, 1)?;
    }

    if version < 2 {
        // Migration 2: enrich citation_relations with match metadata
        let _ = conn.execute(
            "ALTER TABLE citation_relations ADD COLUMN match_status TEXT NOT NULL DEFAULT 'manual'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE citation_relations ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE citation_relations ADD COLUMN raw_ref_text TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE citation_relations ADD COLUMN created_at TEXT NOT NULL DEFAULT (datetime('now'))",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE citation_relations ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'))",
            [],
        );

        // New table: reference_extractions — tracks PDF ref-section extraction attempts
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reference_extractions (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                doc_id              INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                section_text        TEXT NOT NULL,
                extraction_method   TEXT NOT NULL DEFAULT 'heuristic_regex',
                extraction_success  INTEGER NOT NULL DEFAULT 0,
                extracted_at        TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(doc_id)
            );
            CREATE INDEX IF NOT EXISTS idx_ref_ext_doc ON reference_extractions(doc_id);",
        )?;

        // New table: graph_cache — stores rendered citation graphs
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS graph_cache (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key       TEXT NOT NULL,
                graph_data      TEXT NOT NULL,
                edge_version    INTEGER NOT NULL DEFAULT 0,
                doc_count       INTEGER NOT NULL,
                render_mode     TEXT NOT NULL DEFAULT 'visual',
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(cache_key)
            );
            CREATE INDEX IF NOT EXISTS idx_graph_cache_key ON graph_cache(cache_key);",
        )?;

        // Seed citation-related config defaults
        conn.execute_batch(
            "INSERT OR IGNORE INTO app_config (key, value) VALUES ('citation_auto_extract', 'true');
             INSERT OR IGNORE INTO app_config (key, value) VALUES ('citation_fuzzy_threshold', '0.85');
             INSERT OR IGNORE INTO app_config (key, value) VALUES ('citation_override_similarity', 'true');",
        )?;

        set_version(conn, 2)?;
    }

    Ok(())
}
