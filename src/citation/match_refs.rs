use anyhow::Result;
use rusqlite::Connection;
use strsim::jaro_winkler;

use super::extract::ReferenceEntry;

#[derive(Clone, Debug, PartialEq)]
pub enum MatchStatus {
    AutoDoi,
    AutoArxiv,
    AutoTitle,
    AutoFuzzy,
    Manual,
    BibtexImport,
}

impl MatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchStatus::AutoDoi => "auto_doi",
            MatchStatus::AutoArxiv => "auto_arxiv",
            MatchStatus::AutoTitle => "auto_title",
            MatchStatus::AutoFuzzy => "auto_fuzzy",
            MatchStatus::Manual => "manual",
            MatchStatus::BibtexImport => "bibtex_import",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "auto_doi" => Some(MatchStatus::AutoDoi),
            "auto_arxiv" => Some(MatchStatus::AutoArxiv),
            "auto_title" => Some(MatchStatus::AutoTitle),
            "auto_fuzzy" => Some(MatchStatus::AutoFuzzy),
            "manual" => Some(MatchStatus::Manual),
            "bibtex_import" => Some(MatchStatus::BibtexImport),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchResult {
    pub doc_id: i64,
    pub match_status: MatchStatus,
    pub confidence: f64,
}

const _FUZZY_THRESHOLD: f64 = 0.85;

pub fn match_reference_to_doc(
    conn: &Connection,
    r: &ReferenceEntry,
    fuzzy_threshold: f64,
) -> Result<Option<MatchResult>> {
    if let Some(ref doi) = r.doi {
        if let Some(id) = find_by_doi(conn, doi)? {
            return Ok(Some(MatchResult {
                doc_id: id,
                match_status: MatchStatus::AutoDoi,
                confidence: 1.0,
            }));
        }
    }

    if let Some(ref arxiv_id) = r.arxiv_id {
        if let Some(id) = find_by_arxiv_id(conn, arxiv_id)? {
            return Ok(Some(MatchResult {
                doc_id: id,
                match_status: MatchStatus::AutoArxiv,
                confidence: 1.0,
            }));
        }
    }

    if let Some(ref title) = r.title {
        if let Some(id) = find_by_title_year(conn, title, r.pub_year)? {
            return Ok(Some(MatchResult {
                doc_id: id,
                match_status: MatchStatus::AutoTitle,
                confidence: 1.0,
            }));
        }

        let fuzzy = find_by_fuzzy_title(conn, title, fuzzy_threshold)?;
        if let Some((id, score)) = fuzzy {
            return Ok(Some(MatchResult {
                doc_id: id,
                match_status: MatchStatus::AutoFuzzy,
                confidence: score,
            }));
        }
    }

    Ok(None)
}

fn find_by_doi(conn: &Connection, doi: &str) -> Result<Option<i64>> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM documents WHERE doi = ?1 LIMIT 1",
            rusqlite::params![doi],
            |row| row.get(0),
        )
        .ok();
    Ok(id)
}

fn find_by_arxiv_id(conn: &Connection, arxiv_id: &str) -> Result<Option<i64>> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM documents WHERE arxiv_id = ?1 LIMIT 1",
            rusqlite::params![arxiv_id],
            |row| row.get(0),
        )
        .ok();
    Ok(id)
}

fn find_by_title_year(conn: &Connection, title: &str, year: Option<i64>) -> Result<Option<i64>> {
    let norm = normalize_title(title);
    let id: Option<i64> = if let Some(y) = year {
        conn.query_row(
            "SELECT id FROM documents WHERE pub_year = ?1 LIMIT 1",
            rusqlite::params![y],
            |row| row.get(0),
        )
        .ok()
        .and_then(|candidate_id| verify_title_match(conn, candidate_id, &norm))
    } else {
        let mut stmt = conn.prepare("SELECT id, title FROM documents")?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let t: String = row.get(1)?;
            Ok((id, t))
        })?;
        rows.filter_map(|r| r.ok())
            .find(|(_, t)| normalize_title(t) == norm)
            .map(|(id, _)| id)
    };
    Ok(id)
}

fn verify_title_match(conn: &Connection, doc_id: i64, norm_query: &str) -> Option<i64> {
    let title: String = conn
        .query_row(
            "SELECT title FROM documents WHERE id = ?1",
            rusqlite::params![doc_id],
            |row| row.get(0),
        )
        .ok()?;
    if normalize_title(&title) == norm_query {
        Some(doc_id)
    } else {
        None
    }
}

fn find_by_fuzzy_title(
    conn: &Connection,
    title: &str,
    threshold: f64,
) -> Result<Option<(i64, f64)>> {
    let norm_query = normalize_title(title);
    let mut stmt = conn.prepare("SELECT id, title FROM documents")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let t: String = row.get(1)?;
        Ok((id, t))
    })?;

    let mut best: Option<(i64, f64)> = None;
    for row in rows {
        let (id, t) = match row {
            Ok(r) => r,
            Err(_) => continue,
        };
        let norm_t = normalize_title(&t);
        let score = jaro_winkler(&norm_query, &norm_t);
        if score >= threshold {
            match best {
                Some((_, best_score)) if score <= best_score => {}
                _ => best = Some((id, score)),
            }
        }
    }
    Ok(best)
}

pub fn normalize_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let stripped: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect();
    let words: Vec<&str> = stripped
        .split_whitespace()
        .filter(|w| *w != "the" && *w != "a" && *w != "an")
        .collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_strips_articles_and_punctuation() {
        assert_eq!(normalize_title("The Quick, Brown Fox!"), "quick brown fox");
    }

    #[test]
    fn test_normalize_title_case_insensitive() {
        assert_eq!(
            normalize_title("Deep Learning"),
            normalize_title("deep learning")
        );
    }

    #[test]
    fn test_match_status_roundtrip() {
        let statuses = [
            MatchStatus::AutoDoi,
            MatchStatus::AutoArxiv,
            MatchStatus::AutoTitle,
            MatchStatus::AutoFuzzy,
            MatchStatus::Manual,
            MatchStatus::BibtexImport,
        ];
        for s in &statuses {
            assert_eq!(MatchStatus::from_str(s.as_str()), Some(s.clone()));
        }
    }
}
