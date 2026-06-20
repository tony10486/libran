pub mod documents;
pub mod facets;
pub mod migrations;
pub mod projects;
pub mod schema;
pub mod search;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbConn = Arc<Mutex<Connection>>;

pub fn init_database(conn: &Connection) -> Result<()> {
    schema::create_tables(conn)?;
    migrations::run(conn)?;
    Ok(())
}

pub fn open_database(path: &Path) -> Result<DbConn> {
    if let Some(parent) = path.parent()
        && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    init_database(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}
