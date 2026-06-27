use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::custom::CustomScheme;
use super::scheme::{ClassificationNode, SchemeCode, register_scheme};

/// Import a classification scheme from a CSV file.
///
/// CSV format: `notation,pref_label,broader_notation,alt_labels,notes`
/// - `broader_notation`: optional (empty = root node)
/// - `alt_labels`: semicolon-separated, optional
/// - `notes`: optional (stored as scope_note)
///
/// The scheme code is derived from the filename stem (e.g. `my-scheme.csv` -> `my-scheme`).
/// Returns the `scheme_id` of the registered scheme.
pub fn import_classification_csv(conn: &Connection, path: &Path) -> Result<i64> {
    let code = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("custom")
        .to_string();

    let csv_content = std::fs::read_to_string(path)
        .with_context(|| format!("CSV 파일을 열 수 없습니다: {}", path.display()))?;

    import_from_csv_str(conn, &code, &csv_content)
}

/// Import a classification scheme from a CSV string.
///
/// Parses the CSV content, builds a `CustomScheme`, and registers it via `register_scheme`.
/// The `code` is used as both the scheme code and the scheme name.
pub fn import_from_csv_str(conn: &Connection, code: &str, csv_content: &str) -> Result<i64> {
    let nodes = parse_csv_nodes(code, csv_content)?;
    let scheme = CustomScheme::new(code, code, nodes);
    register_scheme(conn, &scheme)
}

fn parse_csv_nodes(scheme_code: &str, csv_content: &str) -> Result<Vec<ClassificationNode>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut nodes = Vec::new();
    for (idx, record) in reader.records().enumerate() {
        let record = record.with_context(|| format!("CSV {}번째 데이터 줄 파싱 실패", idx + 2))?;

        let notation = record.get(0).unwrap_or("").trim().to_string();
        if notation.is_empty() {
            continue;
        }
        let pref_label = record.get(1).unwrap_or("").trim().to_string();
        let broader = record.get(2).unwrap_or("").trim();
        let alt_labels = record.get(3).unwrap_or("").trim();
        let notes = record.get(4).unwrap_or("").trim();

        let parent_notation = if broader.is_empty() {
            None
        } else {
            Some(broader.to_string())
        };
        let alt_label = if alt_labels.is_empty() {
            None
        } else {
            Some(alt_labels.to_string())
        };
        let scope_note = if notes.is_empty() {
            None
        } else {
            Some(notes.to_string())
        };

        nodes.push(ClassificationNode {
            id: None,
            scheme_code: SchemeCode::Custom(scheme_code.to_string()),
            notation,
            pref_label,
            alt_label,
            scope_note,
            parent_notation,
            sort_order: idx as i64,
        });
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_database(&conn).unwrap();
        conn
    }

    fn node_count(conn: &Connection, scheme_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM classification_nodes WHERE scheme_id = ?1",
            rusqlite::params![scheme_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn root_count(conn: &Connection, scheme_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM classification_nodes WHERE scheme_id = ?1 AND parent_id IS NULL",
            rusqlite::params![scheme_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn parent_of(conn: &Connection, scheme_id: i64, notation: &str) -> Option<i64> {
        conn.query_row(
            "SELECT parent_id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2",
            rusqlite::params![scheme_id, notation],
            |row| row.get(0),
        )
        .ok()
        .flatten()
    }

    fn node_id(conn: &Connection, scheme_id: i64, notation: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2",
            rusqlite::params![scheme_id, notation],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_csv_import_flat() {
        let conn = setup_db();
        let csv = "notation,pref_label,broader_notation,alt_labels,notes\n\
                   A,Alpha,,,note A\n\
                   B,Beta,,,note B\n\
                   C,Charlie,,,note C\n";
        let scheme_id = import_from_csv_str(&conn, "flat", csv).unwrap();

        assert_eq!(node_count(&conn, scheme_id), 3);
        assert_eq!(root_count(&conn, scheme_id), 3);
    }

    #[test]
    fn test_csv_import_hierarchical() {
        let conn = setup_db();
        let csv = "notation,pref_label,broader_notation,alt_labels,notes\n\
                   1,Root,,,root note\n\
                   1.1,Child A,1,,child note\n\
                   1.2,Child B,1,,\n\
                   1.2.1,Grandchild,1.2,,\n";
        let scheme_id = import_from_csv_str(&conn, "hier", csv).unwrap();

        assert_eq!(node_count(&conn, scheme_id), 4);
        assert_eq!(root_count(&conn, scheme_id), 1);

        let root_id = node_id(&conn, scheme_id, "1");
        let child_b_id = node_id(&conn, scheme_id, "1.2");
        let grandchild_parent = parent_of(&conn, scheme_id, "1.2.1");

        assert_eq!(parent_of(&conn, scheme_id, "1.1"), Some(root_id));
        assert_eq!(parent_of(&conn, scheme_id, "1.2"), Some(root_id));
        assert_eq!(grandchild_parent, Some(child_b_id));
    }

    #[test]
    fn test_csv_import_duplicate_notation() {
        let conn = setup_db();
        let csv = "notation,pref_label,broader_notation,alt_labels,notes\n\
                   X,First,,,first\n\
                   X,Second,,,second\n\
                   Y,Other,,,other\n";
        let scheme_id = import_from_csv_str(&conn, "dup", csv).unwrap();

        // INSERT OR IGNORE in register_scheme keeps only the first occurrence per notation
        assert_eq!(node_count(&conn, scheme_id), 2);

        let label: String = conn
            .query_row(
                "SELECT pref_label FROM classification_nodes WHERE scheme_id = ?1 AND notation = 'X'",
                rusqlite::params![scheme_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(label, "First");
    }
}
