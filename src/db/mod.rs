pub mod documents;
pub mod facets;
pub mod migrations;
pub mod projects;
pub mod schema;
pub mod search;

use anyhow::Result;
use rusqlite::Connection;

pub fn init_database(conn: &Connection) -> Result<()> {
    schema::create_tables(conn)?;
    migrations::run(conn)?;
    Ok(())
}
