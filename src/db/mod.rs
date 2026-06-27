pub mod attachments;
pub mod author_aliases;
pub mod backup;
pub mod creators;
pub mod custom_fields;
pub mod documents;
pub mod documents_body;
pub mod facets;
pub mod fts_query;
pub mod migrations;
pub mod notes;
pub mod projects;
pub mod saved_searches;
pub mod schema;
pub mod search;
pub mod series;
pub mod stats;

#[cfg(test)]
mod test_support;

use anyhow::Result;
use rusqlite::Connection;
use rusqlite::functions::FunctionFlags;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbConn = Arc<Mutex<Connection>>;

pub fn init_database(conn: &Connection) -> Result<()> {
    register_fts_functions(conn)?;
    schema::create_tables(conn)?;
    migrations::run(conn)?;
    Ok(())
}

fn register_fts_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "bigrams_cjk",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let s: Option<String> = ctx.get(0)?;
            Ok(s.map(|s| fts_query::bigrams_cjk(&s)).unwrap_or_default())
        },
    )?;

    conn.create_scalar_function(
        "choseong_bigrams_cjk",
        1,
        FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let s: Option<String> = ctx.get(0)?;
            Ok(s.map(|s| fts_query::choseong_bigrams_cjk(&s))
                .unwrap_or_default())
        },
    )?;

    Ok(())
}

pub fn open_database(path: &Path) -> Result<DbConn> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    init_database(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}
