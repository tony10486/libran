use anyhow::Result;
use rusqlite::Connection;

use super::match_refs::MatchStatus;

pub fn add_manual_citation(conn: &Connection, source_id: i64, target_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO citation_relations (citing_id, cited_id, match_status, confidence, created_at, updated_at)
         VALUES (?1, ?2, 'manual', 1.0, datetime('now'), datetime('now'))",
        rusqlite::params![source_id, target_id],
    )?;
    Ok(())
}

pub fn add_bibtex_citation(conn: &Connection, source_id: i64, target_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO citation_relations (citing_id, cited_id, match_status, confidence, created_at, updated_at)
         VALUES (?1, ?2, 'bibtex_import', 1.0, datetime('now'), datetime('now'))",
        rusqlite::params![source_id, target_id],
    )?;
    Ok(())
}

pub fn add_extracted_citation(
    conn: &Connection,
    source_id: i64,
    target_id: i64,
    match_status: &MatchStatus,
    confidence: f64,
    raw_ref_text: Option<&str>,
) -> Result<()> {
    let status_str = match_status.as_str();
    conn.execute(
        "INSERT OR IGNORE INTO citation_relations (citing_id, cited_id, match_status, confidence, raw_ref_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
        rusqlite::params![source_id, target_id, status_str, confidence, raw_ref_text],
    )?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct BibtexEntry {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub year: Option<i64>,
    pub doi: Option<String>,
    pub journal: Option<String>,
    pub key: Option<String>,
}

pub fn parse_bibtex(content: &str) -> Vec<BibtexEntry> {
    let mut entries = Vec::new();

    for entry_block in split_bibtex_entries(content) {
        let mut entry = BibtexEntry {
            title: None,
            authors: None,
            year: None,
            doi: None,
            journal: None,
            key: None,
        };

        for (name, value) in parse_bibtex_fields(&entry_block) {
            apply_bibtex_field(&mut entry, &name, &value);
        }

        entries.push(entry);
    }

    entries
}

fn split_bibtex_entries(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut depth: usize = 0;
    let mut in_entry = false;

    for ch in content.chars() {
        if !in_entry && ch == '@' {
            in_entry = true;
            depth = 0;
            current.clear();
        }
        if in_entry {
            current.push(ch);
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        blocks.push(current.clone());
                        in_entry = false;
                    }
                }
                _ => {}
            }
        }
    }

    blocks
}

fn parse_bibtex_fields(block: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut depth: usize = 0;
    let mut name_buf = String::new();
    let mut value_buf = String::new();
    let mut state: ParseState = ParseState::BeforeEntry;

    enum ParseState {
        BeforeEntry,
        InName,
        AfterName,
        InValue,
    }

    for ch in block.chars() {
        match state {
            ParseState::BeforeEntry => {
                if ch == '{' {
                    depth += 1;
                    state = ParseState::InName;
                }
            }
            ParseState::InName => {
                if ch == '=' {
                    state = ParseState::AfterName;
                } else if ch == ',' || ch == '}' || ch == '\n' {
                    name_buf.clear();
                } else if !ch.is_whitespace() {
                    name_buf.push(ch);
                }
            }
            ParseState::AfterName => {
                if ch == '{' {
                    depth += 1;
                    state = ParseState::InValue;
                    value_buf.clear();
                } else if ch == '"' {
                    state = ParseState::InValue;
                    value_buf.clear();
                } else if !ch.is_whitespace() && ch.is_ascii_digit() {
                    value_buf.push(ch);
                    state = ParseState::InValue;
                }
            }
            ParseState::InValue => match ch {
                '}' => {
                    if depth > 1 {
                        depth -= 1;
                        value_buf.push(ch);
                    } else if depth == 1 {
                        depth -= 1;
                        fields.push((name_buf.trim().to_lowercase(), value_buf.trim().to_string()));
                        name_buf.clear();
                        value_buf.clear();
                        state = ParseState::InName;
                    }
                }
                '"' => {
                    fields.push((name_buf.trim().to_lowercase(), value_buf.trim().to_string()));
                    name_buf.clear();
                    value_buf.clear();
                    state = ParseState::InName;
                }
                ',' if depth <= 1 => {
                    fields.push((name_buf.trim().to_lowercase(), value_buf.trim().to_string()));
                    name_buf.clear();
                    value_buf.clear();
                    state = ParseState::InName;
                }
                '{' => {
                    depth += 1;
                    value_buf.push(ch);
                }
                _ => {
                    value_buf.push(ch);
                }
            },
        }
    }

    if !name_buf.is_empty() && !value_buf.is_empty() {
        fields.push((name_buf.trim().to_lowercase(), value_buf.trim().to_string()));
    }

    fields
}

fn apply_bibtex_field(entry: &mut BibtexEntry, name: &str, value: &str) {
    let cleaned = value
        .trim()
        .trim_matches('"')
        .trim_matches('{')
        .trim_matches('}')
        .trim()
        .trim_end_matches(',');
    if cleaned.is_empty() {
        return;
    }
    match name {
        "title" => entry.title = Some(cleaned.to_string()),
        "author" => entry.authors = Some(cleaned.to_string()),
        "year" => entry.year = cleaned.parse::<i64>().ok(),
        "doi" => entry.doi = Some(cleaned.to_string()),
        "journal" | "booktitle" => entry.journal = Some(cleaned.to_string()),
        _ => {}
    }
}

pub fn match_bibtex_entry(
    conn: &Connection,
    entry: &BibtexEntry,
    fuzzy_threshold: f64,
) -> Result<Option<i64>> {
    let ref_entry = super::extract::ReferenceEntry {
        raw_text: String::new(),
        doi: entry.doi.clone(),
        arxiv_id: None,
        title: entry.title.clone(),
        authors: entry.authors.clone(),
        pub_year: entry.year,
        journal: entry.journal.clone(),
    };
    let result = super::match_refs::match_reference_to_doc(conn, &ref_entry, fuzzy_threshold)?;
    Ok(result.map(|r| r.doc_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bibtex_simple() {
        let content = r#"
@article{smith2023,
  title = "Deep Learning for Science",
  author = "Smith, John and Lee, Jane",
  year = 2023,
  doi = "10.1234/test",
  journal = "Nature"
}
"#;
        let entries = parse_bibtex(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].title.as_deref(),
            Some("Deep Learning for Science")
        );
        assert_eq!(entries[0].year, Some(2023));
        assert_eq!(entries[0].doi.as_deref(), Some("10.1234/test"));
    }

    #[test]
    fn test_parse_bibtex_multiple() {
        let content = r#"
@article{a, title = "First", year = 2021}
@inproceedings{b, title = "Second", year = 2022, booktitle = "ICML"}
"#;
        let entries = parse_bibtex(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title.as_deref(), Some("First"));
        assert_eq!(entries[1].journal.as_deref(), Some("ICML"));
    }
}
