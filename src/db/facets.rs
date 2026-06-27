use anyhow::Result;
use rusqlite::Connection;

use crate::db::fts_query::{SearchPlan, build_search_plan, escape_like, normalize_nfc};

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
    let search_term = search_term
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_nfc(s.trim()));
    let has_search = search_term.is_some();

    let search_plan = search_term.as_deref().map(build_search_plan);

    let sql = build_facet_query(has_project, search_plan.as_ref());
    let mut stmt = conn.prepare(&sql)?;

    let rows = if has_project && has_search {
        let pid = project_id.unwrap();
        match search_plan.as_ref().unwrap() {
            SearchPlan::FtsMatch(escaped) => {
                stmt.query_map(rusqlite::params![pid, escaped], map_facet_row)?
            }
            SearchPlan::BigramMatch(escaped) => {
                stmt.query_map(rusqlite::params![pid, escaped], map_facet_row)?
            }
            SearchPlan::ChoseongMatch(escaped) => {
                stmt.query_map(rusqlite::params![pid, escaped], map_facet_row)?
            }
            SearchPlan::Like(t) => {
                let pattern = format!("%{}%", escape_like(t));
                stmt.query_map(rusqlite::params![pid, pattern], map_facet_row)?
            }
        }
    } else if has_project {
        let pid = project_id.unwrap();
        stmt.query_map(rusqlite::params![pid], map_facet_row)?
    } else if has_search {
        match search_plan.as_ref().unwrap() {
            SearchPlan::FtsMatch(escaped) => {
                stmt.query_map(rusqlite::params![escaped], map_facet_row)?
            }
            SearchPlan::BigramMatch(escaped) => {
                stmt.query_map(rusqlite::params![escaped], map_facet_row)?
            }
            SearchPlan::ChoseongMatch(escaped) => {
                stmt.query_map(rusqlite::params![escaped], map_facet_row)?
            }
            SearchPlan::Like(t) => {
                let pattern = format!("%{}%", escape_like(t));
                stmt.query_map(rusqlite::params![pattern], map_facet_row)?
            }
        }
    } else {
        stmt.query_map([], map_facet_row)?
    };

    let mut facets = Vec::new();
    for row in rows {
        facets.push(row?);
    }
    Ok(facets)
}

fn build_facet_query(has_project: bool, search_plan: Option<&SearchPlan>) -> String {
    let mut sql = String::from(
        "SELECT cs.code, cn.notation, cn.pref_label, COUNT(dc.document_id) as cnt
         FROM document_classifications dc
         INNER JOIN classification_nodes cn ON dc.node_id = cn.id
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         INNER JOIN documents d ON dc.document_id = d.id
         WHERE cs.enabled = 1",
    );

    if has_project {
        sql.push_str(
            " AND d.id IN (SELECT document_id FROM project_documents WHERE project_id = ?1)",
        );
    }

    if let Some(plan) = search_plan {
        let param = if has_project { "?2" } else { "?1" };
        match plan {
            SearchPlan::FtsMatch(_) => {
                sql.push_str(&format!(
                    " AND d.id IN (SELECT rowid FROM documents_fts WHERE documents_fts MATCH {})",
                    param
                ));
            }
            SearchPlan::BigramMatch(_) => {
                sql.push_str(&format!(
                    " AND d.id IN (SELECT rowid FROM documents_bigram_fts WHERE documents_bigram_fts MATCH {})",
                    param
                ));
            }
            SearchPlan::ChoseongMatch(_) => {
                sql.push_str(&format!(
                    " AND d.id IN (SELECT rowid FROM documents_choseong_fts WHERE documents_choseong_fts MATCH {})",
                    param
                ));
            }
            SearchPlan::Like(_) => {
                sql.push_str(&format!(
                    " AND (d.title LIKE {} ESCAPE '\\'
                       OR d.authors LIKE {} ESCAPE '\\'
                       OR d.journal LIKE {} ESCAPE '\\'
                       OR d.abstract LIKE {} ESCAPE '\\'
                       OR d.keywords LIKE {} ESCAPE '\\')",
                    param, param, param, param, param
                ));
            }
        }
    }

    sql.push_str(
        " GROUP BY cs.code, cn.notation, cn.pref_label ORDER BY cnt DESC, cn.notation ASC",
    );
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
