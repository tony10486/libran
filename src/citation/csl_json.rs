use crate::db::documents::Document;
use crate::export::fetch_user_export_data;
use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct CslItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    title: String,
    author: Vec<CslAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "container-title")]
    container_title: Option<String>,
    issued: CslDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(rename = "arxiv-id")]
    arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    category: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CslAuthor {
    family: String,
    given: String,
}

#[derive(Serialize, Deserialize)]
struct CslDate {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<i64>>,
}

fn csl_type_from_item_type(item_type: &str) -> &'static str {
    match item_type {
        "article" => "article-journal",
        "book" => "book",
        "thesis" => "thesis",
        "conference" => "paper-conference",
        "dataset" => "dataset",
        "webpage" => "webpage",
        "patent" => "patent",
        _ => "document",
    }
}

fn item_type_from_csl_type(csl_type: &str) -> String {
    match csl_type {
        "article-journal" => "article",
        "book" => "book",
        "thesis" => "thesis",
        "paper-conference" => "conference",
        "dataset" => "dataset",
        "webpage" => "webpage",
        "patent" => "patent",
        _ => "misc",
    }
    .to_string()
}

pub fn export_csl_json(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let items: Vec<CslItem> = documents
        .iter()
        .map(|doc| {
            let authors = doc
                .authors
                .as_ref()
                .map(|a| {
                    a.split(';')
                        .filter_map(|name| {
                            let name = name.trim();
                            if name.is_empty() {
                                return None;
                            }
                            if let Some(comma_pos) = name.find(',') {
                                Some(CslAuthor {
                                    family: name[..comma_pos].trim().to_string(),
                                    given: name[comma_pos + 1..].trim().to_string(),
                                })
                            } else {
                                let parts: Vec<&str> = name.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let (given, family) = parts.split_at(parts.len() - 1);
                                    Some(CslAuthor {
                                        family: family[0].to_string(),
                                        given: given.join(" "),
                                    })
                                } else {
                                    Some(CslAuthor {
                                        family: name.to_string(),
                                        given: String::new(),
                                    })
                                }
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let year = doc.pub_year.unwrap_or(0);
            let date_parts = if year > 0 {
                vec![vec![year]]
            } else {
                vec![vec![]]
            };

            CslItem {
                id: doc
                    .citation_key
                    .clone()
                    .unwrap_or_else(|| format!("doc{}", doc.id.unwrap_or(0))),
                item_type: csl_type_from_item_type(&doc.item_type).to_string(),
                title: doc.title.clone(),
                author: authors,
                container_title: doc.journal.clone(),
                issued: CslDate { date_parts },
                doi: doc.doi.clone(),
                arxiv_id: doc.arxiv_id.clone(),
                keyword: doc.keywords.clone(),
                note: None,
                category: None,
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&items)?;
    writer.write_all(json.as_bytes())?;
    Ok(())
}

pub fn export_csl_json_with_user_data(
    conn: &Connection,
    documents: &[Document],
    writer: &mut impl Write,
) -> Result<()> {
    let items: Vec<CslItem> = documents
        .iter()
        .map(|doc| {
            let user_data = doc
                .id
                .and_then(|id| fetch_user_export_data(conn, id).ok())
                .unwrap_or_default();

            let authors = doc
                .authors
                .as_ref()
                .map(|a| {
                    a.split(';')
                        .filter_map(|name| {
                            let name = name.trim();
                            if name.is_empty() {
                                return None;
                            }
                            if let Some(comma_pos) = name.find(',') {
                                Some(CslAuthor {
                                    family: name[..comma_pos].trim().to_string(),
                                    given: name[comma_pos + 1..].trim().to_string(),
                                })
                            } else {
                                let parts: Vec<&str> = name.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let (given, family) = parts.split_at(parts.len() - 1);
                                    Some(CslAuthor {
                                        family: family[0].to_string(),
                                        given: given.join(" "),
                                    })
                                } else {
                                    Some(CslAuthor {
                                        family: name.to_string(),
                                        given: String::new(),
                                    })
                                }
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let year = doc.pub_year.unwrap_or(0);
            let date_parts = if year > 0 {
                vec![vec![year]]
            } else {
                vec![vec![]]
            };

            let keyword = {
                let mut parts: Vec<&str> = doc
                    .keywords
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .into_iter()
                    .flat_map(|s| s.split(',').map(str::trim))
                    .filter(|s| !s.is_empty())
                    .collect();
                parts.extend(user_data.tags.iter().map(String::as_str));
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(", "))
                }
            };

            let note = if user_data.notes.is_empty() {
                None
            } else {
                Some(user_data.notes.join("\n\n"))
            };

            let category = if user_data.classifications.is_empty() {
                None
            } else {
                Some(user_data.classifications.join(", "))
            };

            CslItem {
                id: doc
                    .citation_key
                    .clone()
                    .unwrap_or_else(|| format!("doc{}", doc.id.unwrap_or(0))),
                item_type: csl_type_from_item_type(&doc.item_type).to_string(),
                title: doc.title.clone(),
                author: authors,
                container_title: doc.journal.clone(),
                issued: CslDate { date_parts },
                doi: doc.doi.clone(),
                arxiv_id: doc.arxiv_id.clone(),
                keyword,
                note,
                category,
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&items)?;
    writer.write_all(json.as_bytes())?;
    Ok(())
}

pub fn parse_csl_json(json: &str) -> Result<Vec<Document>> {
    let items: Vec<CslItem> = serde_json::from_str(json)?;
    let documents = items
        .into_iter()
        .map(|item| {
            let authors = if item.author.is_empty() {
                None
            } else {
                let joined = item
                    .author
                    .iter()
                    .map(|a| {
                        if a.given.is_empty() {
                            a.family.clone()
                        } else {
                            format!("{}, {}", a.family, a.given)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                Some(joined)
            };

            let pub_year = item.issued.date_parts.into_iter().flatten().next();

            Document {
                id: None,
                title: item.title,
                authors,
                journal: item.container_title,
                conference: None,
                pub_year,
                doi: item.doi,
                arxiv_id: item.arxiv_id,
                abstract_text: None,
                keywords: item.keyword,
                file_path: None,
                file_hash: None,
                citation_key: Some(item.id),
                source: None,
                rating: None,
                item_type: item_type_from_csl_type(&item.item_type),
                ..Default::default()
            }
        })
        .collect();
    Ok(documents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation::bibtex::export_bibtex;
    use std::io::Cursor;

    #[test]
    fn test_round_trip_bibtex_to_csl_to_bibtex() {
        let original = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            keywords: Some("AI, ML".to_string()),
            citation_key: Some("smith2023".to_string()),
            ..Default::default()
        };

        let mut bib_buf1 = Vec::new();
        export_bibtex(&[original.clone()], &mut Cursor::new(&mut bib_buf1)).unwrap();
        let bib1 = String::from_utf8(bib_buf1).unwrap();

        let mut csl_buf = Vec::new();
        export_csl_json(&[original.clone()], &mut Cursor::new(&mut csl_buf)).unwrap();
        let csl_json = String::from_utf8(csl_buf).unwrap();

        let parsed = parse_csl_json(&csl_json).unwrap();
        assert_eq!(parsed.len(), 1);

        let mut bib_buf2 = Vec::new();
        export_bibtex(&parsed, &mut Cursor::new(&mut bib_buf2)).unwrap();
        let bib2 = String::from_utf8(bib_buf2).unwrap();

        assert!(bib2.contains("Deep Learning"), "title preserved: {bib2}");
        assert!(
            bib2.contains("Smith, John"),
            "first author preserved: {bib2}"
        );
        assert!(
            bib2.contains("Lee, Jane"),
            "second author preserved: {bib2}"
        );
        assert!(bib2.contains("Nature"), "journal preserved: {bib2}");
        assert!(bib2.contains("2023"), "year preserved: {bib2}");
        assert!(bib2.contains("10.1234/test"), "DOI preserved: {bib2}");
        assert!(bib2.contains("smith2023"), "citation key preserved: {bib2}");
    }

    #[test]
    fn test_parse_csl_json_single_author_no_comma() {
        let json = r#" [{
            "id": "key1",
            "type": "article-journal",
            "title": "Test",
            "author": [{"family": "Smith", "given": "John"}],
            "issued": {"date-parts": [[2023]]},
            "container-title": "Nature"
        }] "#;
        let docs = parse_csl_json(json).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].authors, Some("Smith, John".to_string()));
        assert_eq!(docs[0].pub_year, Some(2023));
    }

    #[test]
    fn test_parse_csl_json_no_authors() {
        let json = r#" [{
            "id": "key2",
            "type": "document",
            "title": "Untitled",
            "author": [],
            "issued": {"date-parts": [[]]}
        }] "#;
        let docs = parse_csl_json(json).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].authors, None);
        assert_eq!(docs[0].pub_year, None);
    }
}
