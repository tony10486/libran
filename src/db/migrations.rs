use anyhow::Result;
use rusqlite::Connection;

use crate::db::fts_query::normalize_nfc;

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

    if version < 3 {
        // Migration 3: populate bigram FTS table + NFC-normalize existing documents
        conn.execute(
            "INSERT INTO documents_bigram_fts(rowid, title, authors, journal, abstract, keywords)
             SELECT id, bigrams_cjk(title), bigrams_cjk(authors), bigrams_cjk(journal),
                    bigrams_cjk(abstract), bigrams_cjk(keywords)
             FROM documents",
            [],
        )?;

        nfc_normalize_existing_documents(conn)?;

        set_version(conn, 3)?;
    }

    if version < 4 {
        // Migration 4: populate choseong FTS table for 초성 search
        conn.execute(
            "INSERT INTO documents_choseong_fts(rowid, title, authors, journal, abstract, keywords)
             SELECT id, choseong_bigrams_cjk(title), choseong_bigrams_cjk(authors),
                    choseong_bigrams_cjk(journal), choseong_bigrams_cjk(abstract),
                    choseong_bigrams_cjk(keywords)
             FROM documents",
            [],
        )?;

        set_version(conn, 4)?;
    }

    if version < 5 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS document_notes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL UNIQUE,
                content     TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
            );",
        )?;
        set_version(conn, 5)?;
    }

    if version < 6 {
        // Migration 6: add rating column to documents (nullable, 1-5 stars)
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN rating INTEGER", []);
        set_version(conn, 6)?;
    }

    if version < 7 {
        // Migration 7: series bundling — series + document_series tables
        // Tables are CREATE IF NOT EXISTS in schema.rs, so they already exist for
        // fresh databases. For pre-existing DBs the migration guarantees presence
        // and seeds the series auto-grouping config default.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS series (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                publisher       TEXT,
                issn            TEXT,
                created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_series_name ON series(name);
            CREATE INDEX IF NOT EXISTS idx_series_issn ON series(issn);

            CREATE TABLE IF NOT EXISTS document_series (
                document_id     INTEGER NOT NULL,
                series_id       INTEGER NOT NULL,
                volume          TEXT,
                issue           TEXT,
                sort_order      INTEGER DEFAULT 0,
                added_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (document_id, series_id),
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
                FOREIGN KEY (series_id) REFERENCES series(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_doc_series_series ON document_series(series_id);
            CREATE INDEX IF NOT EXISTS idx_doc_series_doc ON document_series(document_id);

            INSERT OR IGNORE INTO app_config (key, value) VALUES ('series_grouping_enabled', 'false');",
        )?;
        set_version(conn, 7)?;
    }

    if version < 8 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS document_custom_fields (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id     INTEGER NOT NULL,
                field_key       TEXT NOT NULL,
                field_value     TEXT,
                created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
                UNIQUE (document_id, field_key)
            );
            CREATE INDEX IF NOT EXISTS idx_custom_fields_doc ON document_custom_fields(document_id);",
        )?;
        set_version(conn, 8)?;
    }

    Ok(())
}

fn nfc_normalize_existing_documents(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, title, authors, journal, abstract, keywords FROM documents",
    )?;
    let docs: Vec<(i64, String, Option<String>, Option<String>, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    for (id, title, authors, journal, abstract_text, keywords) in docs {
        let title_n = normalize_nfc(&title);
        let authors_n = authors.as_deref().map(normalize_nfc);
        let journal_n = journal.as_deref().map(normalize_nfc);
        let abstract_n = abstract_text.as_deref().map(normalize_nfc);
        let keywords_n = keywords.as_deref().map(normalize_nfc);

        let changed = title_n != title
            || authors_n != authors
            || journal_n != journal
            || abstract_n != abstract_text
            || keywords_n != keywords;

        if changed {
            conn.execute(
                "UPDATE documents SET title = ?1, authors = ?2, journal = ?3,
                 abstract = ?4, keywords = ?5 WHERE id = ?6",
                rusqlite::params![
                    title_n,
                    authors_n,
                    journal_n,
                    abstract_n,
                    keywords_n,
                    id,
                ],
            )?;
        }
    }

    Ok(())
}
