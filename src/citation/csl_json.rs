use crate::db::documents::Document;
use anyhow::Result;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct CslItem {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    title: String,
    author: Vec<CslAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "container-title")]
    container_title: Option<String>,
    issued: CslDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "arxiv-id")]
    arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword: Option<String>,
}

#[derive(Serialize)]
struct CslAuthor {
    family: String,
    given: String,
}

#[derive(Serialize)]
struct CslDate {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<i64>>,
}

pub fn export_csl_json(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let items: Vec<CslItem> = documents.iter().map(|doc| {
        let authors = doc.authors.as_ref().map(|a| {
            a.split(';').filter_map(|name| {
                let name = name.trim();
                if name.is_empty() { return None; }
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
            }).collect()
        }).unwrap_or_default();

        let year = doc.pub_year.unwrap_or(0);
        let date_parts = if year > 0 { vec![vec![year]] } else { vec![vec![]] };

        CslItem {
            id: doc.citation_key.clone().unwrap_or_else(|| format!("doc{}", doc.id.unwrap_or(0))),
            item_type: if doc.journal.is_some() { "article-journal".to_string() } else { "document".to_string() },
            title: doc.title.clone(),
            author: authors,
            container_title: doc.journal.clone(),
            issued: CslDate { date_parts },
            doi: doc.doi.clone(),
            arxiv_id: doc.arxiv_id.clone(),
            keyword: doc.keywords.clone(),
        }
    }).collect();

    let json = serde_json::to_string_pretty(&items)?;
    writer.write_all(json.as_bytes())?;
    Ok(())
}
