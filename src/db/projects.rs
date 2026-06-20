use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
}

pub fn create_project(conn: &Connection, name: &str, description: Option<&str>) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (name, description) VALUES (?1, ?2)",
        params![name, description],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT id, name, description FROM projects ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
        })
    })?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

pub fn add_document(conn: &Connection, project_id: i64, document_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO project_documents (project_id, document_id) VALUES (?1, ?2)",
        params![project_id, document_id],
    )?;
    Ok(())
}

pub fn remove_document(conn: &Connection, project_id: i64, document_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM project_documents WHERE project_id = ?1 AND document_id = ?2",
        params![project_id, document_id],
    )?;
    Ok(())
}

pub fn list_documents(conn: &Connection, project_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT document_id FROM project_documents WHERE project_id = ?1 ORDER BY added_at DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}
