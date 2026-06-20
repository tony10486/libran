use anyhow::Result;
use rusqlite::Connection;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SchemeCode {
    Udc,
    Physh,
    Msc,
    Lcc,
    Custom(String),
}

impl SchemeCode {
    pub fn as_str(&self) -> &str {
        match self {
            SchemeCode::Udc => "udc",
            SchemeCode::Physh => "physh",
            SchemeCode::Msc => "msc",
            SchemeCode::Lcc => "lcc",
            SchemeCode::Custom(s) => s.as_str(),
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "udc" => SchemeCode::Udc,
            "physh" => SchemeCode::Physh,
            "msc" => SchemeCode::Msc,
            "lcc" => SchemeCode::Lcc,
            other => SchemeCode::Custom(other.to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClassificationNode {
    pub id: Option<i64>,
    pub scheme_code: SchemeCode,
    pub notation: String,
    pub pref_label: String,
    pub alt_label: Option<String>,
    pub scope_note: Option<String>,
    pub parent_notation: Option<String>,
    pub sort_order: i64,
}

pub trait ClassificationScheme {
    fn code(&self) -> SchemeCode;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn license(&self) -> &str;
    fn source_url(&self) -> &str;
    fn is_primary(&self) -> bool;
    fn nodes(&self) -> &[ClassificationNode];
    fn validate_notation(&self, notation: &str) -> bool;
    fn find_by_notation(&self, notation: &str) -> Option<&ClassificationNode> {
        self.nodes().iter().find(|n| n.notation == notation)
    }
    fn children(&self, parent_notation: &str) -> Vec<&ClassificationNode> {
        self.nodes()
            .iter()
            .filter(|n| n.parent_notation.as_deref() == Some(parent_notation))
            .collect()
    }
    fn search(&self, term: &str) -> Vec<&ClassificationNode> {
        let lower = term.to_lowercase();
        self.nodes()
            .iter()
            .filter(|n| {
                n.pref_label.to_lowercase().contains(&lower)
                    || n
                        .alt_label
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&lower))
                        .unwrap_or(false)
            })
            .collect()
    }
}

pub fn register_scheme(conn: &Connection, scheme: &dyn ClassificationScheme) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO classification_schemes (code, name, version, enabled, is_primary, license, source_url)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6)",
        rusqlite::params![
            scheme.code().as_str(),
            scheme.name(),
            scheme.version(),
            scheme.is_primary() as i64,
            scheme.license(),
            scheme.source_url(),
        ],
    )?;
    let scheme_id: i64 = conn.query_row(
        "SELECT id FROM classification_schemes WHERE code = ?1",
        rusqlite::params![scheme.code().as_str()],
        |row| row.get(0),
    )?;

    for node in scheme.nodes() {
        let parent_id: Option<i64> = if let Some(ref parent_not) = node.parent_notation {
            conn.query_row(
                "SELECT id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2",
                rusqlite::params![scheme_id, parent_not],
                |row| row.get(0),
            )
            .ok()
        } else {
            None
        };

        conn.execute(
            "INSERT OR IGNORE INTO classification_nodes (scheme_id, notation, pref_label, alt_label, scope_note, parent_id, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                scheme_id,
                node.notation,
                node.pref_label,
                node.alt_label,
                node.scope_note,
                parent_id,
                node.sort_order,
            ],
        )?;
    }

    Ok(scheme_id)
}

pub fn assign_classification(
    conn: &Connection,
    document_id: i64,
    node_id: i64,
    is_primary: bool,
    confidence: Option<f64>,
    assigned_by: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO document_classifications (document_id, node_id, is_primary, confidence, assigned_by)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![document_id, node_id, is_primary as i64, confidence, assigned_by],
    )?;
    Ok(())
}

pub fn get_node_id(conn: &Connection, scheme_code: &str, notation: &str) -> Result<Option<i64>> {
    let result: Option<i64> = conn
        .query_row(
            "SELECT cn.id FROM classification_nodes cn
             INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
             WHERE cs.code = ?1 AND cn.notation = ?2",
            rusqlite::params![scheme_code, notation],
            |row| row.get(0),
        )
        .ok();
    Ok(result)
}

pub fn set_label_by_notation(
    conn: &Connection,
    scheme_id: i64,
    notation: &str,
    lang: &str,
    label: &str,
    source: &str,
) -> Result<()> {
    let node_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2",
            rusqlite::params![scheme_id, notation],
            |row| row.get(0),
        )
        .ok();

    if let Some(nid) = node_id {
        conn.execute(
            "INSERT OR REPLACE INTO classification_labels (node_id, lang, label, source)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![nid, lang, label, source],
        )?;
    }
    Ok(())
}
