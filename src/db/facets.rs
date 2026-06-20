use anyhow::Result;
use rusqlite::Connection;

pub struct FacetCount {
    pub scheme_code: String,
    pub notation: String,
    pub pref_label: String,
    pub count: i64,
}

pub fn count_by_classification(
    conn: &Connection,
    project_id: Option<i64>,
    search_term: Option<&str>,
) -> Result<Vec<FacetCount>> {
    let has_project = project_id.is_some();
    let has_search = search_term.map(|s| !s.trim().is_empty()).unwrap_or(false);

    let sql = build_facet_query(has_project, has_search);
    let mut stmt = conn.prepare(&sql)?;

    let rows = if has_project && has_search {
        let pid = project_id.unwrap();
        let escaped = escape_fts_query(search_term.unwrap());
        stmt.query_map(rusqlite::params![pid, escaped], map_facet_row)?
    } else if has_project {
        let pid = project_id.unwrap();
        stmt.query_map(rusqlite::params![pid], map_facet_row)?
    } else if has_search {
        let escaped = escape_fts_query(search_term.unwrap());
        stmt.query_map(rusqlite::params![escaped], map_facet_row)?
    } else {
        stmt.query_map([], map_facet_row)?
    };

    let mut facets = Vec::new();
    for row in rows {
        facets.push(row?);
    }
    Ok(facets)
}

fn build_facet_query(has_project: bool, has_search: bool) -> String {
    let mut sql = String::from(
        "SELECT cs.code, cn.notation, cn.pref_label, COUNT(dc.document_id) as cnt
         FROM document_classifications dc
         INNER JOIN classification_nodes cn ON dc.node_id = cn.id
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         INNER JOIN documents d ON dc.document_id = d.id
         WHERE cs.enabled = 1",
    );

    if has_project {
        sql.push_str(" AND d.id IN (SELECT document_id FROM project_documents WHERE project_id = ?1)");
    }

    if has_search {
        let param = if has_project { "?2" } else { "?1" };
        sql.push_str(&format!(" AND d.id IN (SELECT rowid FROM documents_fts WHERE documents_fts MATCH {})", param));
    }

    sql.push_str(" GROUP BY cs.code, cn.notation, cn.pref_label ORDER BY cnt DESC, cn.notation ASC");
    sql
}

fn map_facet_row(row: &rusqlite::Row) -> rusqlite::Result<FacetCount> {
    Ok(FacetCount {
        scheme_code: row.get(0)?,
        notation: row.get(1)?,
        pref_label: row.get(2)?,
        count: row.get(3)?,
    })
}

fn escape_fts_query(term: &str) -> String {
    if term.len() < 3 {
        return format!("\"{}\"", term);
    }
    format!("\"{}\"", term.replace('"', "\"\""))
}
