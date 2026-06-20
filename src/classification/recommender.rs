use anyhow::Result;
use rusqlite::Connection;

use super::scheme::SchemeCode;
use crate::db::documents::Document;

pub struct Recommendation {
    pub scheme_code: SchemeCode,
    pub notation: String,
    pub pref_label: String,
    pub confidence: f64,
}

pub fn recommend(conn: &Connection, doc: &Document, limit: usize) -> Result<Vec<Recommendation>> {
    let search_text = build_search_text(doc);
    if search_text.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT cs.code, cn.notation, cn.pref_label
         FROM classification_nodes cn
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         WHERE cs.enabled = 1
         ORDER BY LENGTH(cn.notation) ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut scored: Vec<Recommendation> = Vec::new();
    for row in rows {
        let (code, notation, label) = row?;
        let score = score_match(&search_text, &label);
        if score > 0.0 {
            scored.push(Recommendation {
                scheme_code: SchemeCode::parse(&code),
                notation,
                pref_label: label,
                confidence: score,
            });
        }
    }

    scored.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

fn build_search_text(doc: &Document) -> String {
    let mut parts = Vec::new();
    if !doc.title.is_empty() {
        parts.push(doc.title.as_str());
    }
    if let Some(ref j) = doc.journal {
        parts.push(j.as_str());
    }
    if let Some(ref k) = doc.keywords {
        parts.push(k.as_str());
    }
    parts.join(" ").to_lowercase()
}

fn score_match(search_text: &str, label: &str) -> f64 {
    let label_lower = label.to_lowercase();
    let label_words: Vec<&str> = label_lower.split_whitespace().collect();
    if label_words.is_empty() {
        return 0.0;
    }

    let mut matches = 0;
    for word in &label_words {
        if word.len() >= 4 && search_text.contains(word) {
            matches += 1;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let score = matches as f64 / label_words.len() as f64;
    score.min(1.0)
}
