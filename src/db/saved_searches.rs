use anyhow::Result;
use rusqlite::{Connection, params, types::Value};
use serde::{Deserialize, Serialize};

use crate::db::documents::Document;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: i64,
    pub name: String,
    pub fts_query: Option<String>,
    pub filters_json: Option<String>,
    pub created_at: String,
}

/// Join mode for combining search conditions: All = AND, Any = OR.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum JoinMode {
    #[default]
    All,
    Any,
}

/// A single filter condition: field + operator + value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchCondition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Structured search criteria parsed from `filters_json`.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SearchCriteria {
    #[serde(default)]
    pub conditions: Vec<SearchCondition>,
    #[serde(default)]
    pub join_mode: JoinMode,
}

pub fn list(conn: &Connection) -> Result<Vec<SavedSearch>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fts_query, filters_json, created_at FROM saved_searches ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SavedSearch {
            id: row.get(0)?,
            name: row.get(1)?,
            fts_query: row.get(2)?,
            filters_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<SavedSearch>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fts_query, filters_json, created_at FROM saved_searches WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(SavedSearch {
            id: row.get(0)?,
            name: row.get(1)?,
            fts_query: row.get(2)?,
            filters_json: row.get(3)?,
            created_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn insert(
    conn: &Connection,
    name: &str,
    fts_query: Option<&str>,
    filters_json: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO saved_searches (name, fts_query, filters_json) VALUES (?1, ?2, ?3)",
        params![name, fts_query, filters_json],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Criteria-based search ──

const DOC_COLS: &str = "id, title, authors, journal, pub_year, doi, arxiv_id, abstract, \
     keywords, file_path, file_hash, citation_key, source, conference, rating, volume, issue, \
     page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date, \
     reading_status, reading_progress, queue_position, item_type";

fn doc_from_row(row: &rusqlite::Row) -> rusqlite::Result<Document> {
    Ok(Document {
        id: Some(row.get(0)?),
        title: row.get(1)?,
        authors: row.get(2)?,
        journal: row.get(3)?,
        pub_year: row.get(4)?,
        doi: row.get(5)?,
        arxiv_id: row.get(6)?,
        abstract_text: row.get(7)?,
        keywords: row.get(8)?,
        file_path: row.get(9)?,
        file_hash: row.get(10)?,
        citation_key: row.get(11)?,
        source: row.get(12)?,
        conference: row.get(13)?,
        rating: row.get(14)?,
        volume: row.get(15)?,
        issue: row.get(16)?,
        page_start: row.get(17)?,
        page_end: row.get(18)?,
        publisher: row.get(19)?,
        city: row.get(20)?,
        edition: row.get(21)?,
        isbn: row.get(22)?,
        issn: row.get(23)?,
        url: row.get(24)?,
        accessed_date: row.get(25)?,
        reading_status: row.get(26)?,
        reading_progress: row.get(27)?,
        queue_position: row.get(28)?,
        item_type: row.get(29)?,
    })
}

/// Split a `"start-end"` year range string into two integers.
fn parse_year_range(v: &str) -> (i64, i64) {
    let parts: Vec<&str> = v.split('-').collect();
    let start = parts
        .first()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let end = parts
        .get(1)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(9999);
    (start, end)
}

/// Build a SQL WHERE fragment and bound parameters for a single condition.
fn build_condition(condition: &SearchCondition) -> (String, Vec<Value>) {
    let v = &condition.value;
    match condition.field.as_str() {
        "tag" => (
            "id IN (SELECT document_id FROM tags WHERE tag = ?)".into(),
            vec![Value::Text(v.clone())],
        ),
        "year" => (
            "pub_year = ?".into(),
            vec![Value::Integer(v.parse().unwrap_or(0))],
        ),
        "year_range" => {
            let (start, end) = parse_year_range(v);
            (
                "pub_year BETWEEN ? AND ?".into(),
                vec![Value::Integer(start), Value::Integer(end)],
            )
        }
        "reading_status" => ("reading_status = ?".into(), vec![Value::Text(v.clone())]),
        "rating" => {
            let r = v.parse().unwrap_or(0);
            match condition.operator.as_str() {
                "gte" => ("rating >= ?".into(), vec![Value::Integer(r)]),
                "lte" => ("rating <= ?".into(), vec![Value::Integer(r)]),
                _ => ("rating = ?".into(), vec![Value::Integer(r)]),
            }
        }
        "author" => match condition.operator.as_str() {
            "contains" => ("authors LIKE ?".into(), vec![Value::Text(format!("%{v}%"))]),
            _ => ("authors = ?".into(), vec![Value::Text(v.clone())]),
        },
        "journal" => match condition.operator.as_str() {
            "contains" => ("journal LIKE ?".into(), vec![Value::Text(format!("%{v}%"))]),
            _ => ("journal = ?".into(), vec![Value::Text(v.clone())]),
        },
        "classification" => (
            "id IN (SELECT dc.document_id FROM document_classifications dc \
             INNER JOIN classification_nodes cn ON dc.node_id = cn.id \
             WHERE cn.notation = ?)"
                .into(),
            vec![Value::Text(v.clone())],
        ),
        _ => ("1=1".into(), Vec::new()),
    }
}

/// Build a complete WHERE clause from search criteria.
fn build_where(criteria: &SearchCriteria) -> (String, Vec<Value>) {
    let mut clauses = Vec::new();
    let mut all_params = Vec::new();
    for c in &criteria.conditions {
        let (sql, p) = build_condition(c);
        clauses.push(sql);
        all_params.extend(p);
    }
    let joiner = if criteria.join_mode == JoinMode::All {
        " AND "
    } else {
        " OR "
    };
    (clauses.join(joiner), all_params)
}

/// Execute a structured criteria search, returning matching documents.
pub fn execute_search(conn: &Connection, criteria: &SearchCriteria) -> Result<Vec<Document>> {
    if criteria.conditions.is_empty() {
        return crate::db::documents::list_all(conn);
    }
    let (where_sql, params) = build_where(criteria);
    let sql = format!("SELECT {DOC_COLS} FROM documents WHERE {where_sql} ORDER BY id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), doc_from_row)?;
    let mut docs = Vec::new();
    for row in rows {
        docs.push(row?);
    }
    Ok(docs)
}

/// Execute a saved search, parsing `filters_json` criteria when present.
pub fn execute_saved_search(conn: &Connection, search: &SavedSearch) -> Result<Vec<Document>> {
    if let Some(ref json) = search.filters_json
        && !json.is_empty()
        && json != "{}"
    {
        let criteria: SearchCriteria = serde_json::from_str(json)?;
        return execute_search(conn, &criteria);
    }
    crate::db::documents::list_all(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, documents::Document};

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        db::init_database(&conn)?;
        Ok(conn)
    }

    fn make_doc(title: &str, year: Option<i64>) -> Document {
        Document {
            title: title.to_string(),
            pub_year: year,
            ..Default::default()
        }
    }

    #[test]
    fn test_criteria_filter_by_tag() -> Result<()> {
        let conn = setup()?;
        let d1 = db::documents::insert(&conn, &make_doc("Paper 1", Some(2023)))?;
        let d2 = db::documents::insert(&conn, &make_doc("Paper 2", Some(2023)))?;
        let _d3 = db::documents::insert(&conn, &make_doc("Paper 3", Some(2023)))?;

        db::documents::add_tag(&conn, d1, "physics")?;
        db::documents::add_tag(&conn, d2, "physics")?;

        let criteria = SearchCriteria {
            conditions: vec![SearchCondition {
                field: "tag".into(),
                operator: "is".into(),
                value: "physics".into(),
            }],
            join_mode: JoinMode::All,
        };

        let results = execute_search(&conn, &criteria)?;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.title != "Paper 3"));
        Ok(())
    }

    #[test]
    fn test_criteria_year_range() -> Result<()> {
        let conn = setup()?;
        db::documents::insert(&conn, &make_doc("Old", Some(2010)))?;
        db::documents::insert(&conn, &make_doc("Mid", Some(2022)))?;
        db::documents::insert(&conn, &make_doc("New", Some(2025)))?;

        let criteria = SearchCriteria {
            conditions: vec![SearchCondition {
                field: "year_range".into(),
                operator: "is".into(),
                value: "2020-2024".into(),
            }],
            join_mode: JoinMode::All,
        };

        let results = execute_search(&conn, &criteria)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Mid");
        Ok(())
    }

    #[test]
    fn test_criteria_join_any() -> Result<()> {
        let conn = setup()?;
        let d1 = db::documents::insert(&conn, &make_doc("Tagged", Some(2020)))?;
        let _d2 = db::documents::insert(&conn, &make_doc("Untagged", Some(2020)))?;
        db::documents::insert(&conn, &make_doc("Year match", Some(2023)))?;

        db::documents::add_tag(&conn, d1, "x")?;

        // tag="x" OR year=2023 → d1 (tag) + d3 (year), not d2
        let criteria = SearchCriteria {
            conditions: vec![
                SearchCondition {
                    field: "tag".into(),
                    operator: "is".into(),
                    value: "x".into(),
                },
                SearchCondition {
                    field: "year".into(),
                    operator: "is".into(),
                    value: "2023".into(),
                },
            ],
            join_mode: JoinMode::Any,
        };

        let results = execute_search(&conn, &criteria)?;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.title != "Untagged"));
        Ok(())
    }

    #[test]
    fn test_criteria_join_all() -> Result<()> {
        let conn = setup()?;
        let d1 = db::documents::insert(&conn, &make_doc("Both", Some(2023)))?;
        let d2 = db::documents::insert(&conn, &make_doc("Tag only", Some(2020)))?;
        db::documents::insert(&conn, &make_doc("Year only", Some(2023)))?;

        db::documents::add_tag(&conn, d1, "x")?;
        db::documents::add_tag(&conn, d2, "x")?;

        // tag="x" AND year=2023 → only d1
        let criteria = SearchCriteria {
            conditions: vec![
                SearchCondition {
                    field: "tag".into(),
                    operator: "is".into(),
                    value: "x".into(),
                },
                SearchCondition {
                    field: "year".into(),
                    operator: "is".into(),
                    value: "2023".into(),
                },
            ],
            join_mode: JoinMode::All,
        };

        let results = execute_search(&conn, &criteria)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Both");
        Ok(())
    }
}
