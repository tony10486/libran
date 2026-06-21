use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use std::io::Write;

pub fn export_cff(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "cff-version: \"1.2.0\"")?;
    writeln!(
        writer,
        "message: \"If you use this software, please cite it using these metadata.\""
    )?;
    writeln!(writer, "title: \"Libran Bibliography Export\"")?;
    writeln!(writer, "authors:")?;
    writeln!(writer, "  - family-names: \"Libran\"")?;
    writeln!(writer, "    given-names: \"Export\"")?;
    if documents.is_empty() {
        return Ok(());
    }
    writeln!(writer, "references:")?;
    write_references(documents, writer)?;
    Ok(())
}

pub fn export_cff_references(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    if documents.is_empty() {
        return Ok(());
    }
    writeln!(writer, "references:")?;
    write_references(documents, writer)?;
    Ok(())
}

fn write_references(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for doc in documents {
        write_reference(doc, writer)?;
    }
    Ok(())
}

fn write_reference(doc: &Document, writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "  - type: {}", guess_cff_type(doc))?;
    writeln!(writer, "    title: {}", yaml_quote(&doc.title))?;

    if let Some(authors) = &doc.authors {
        let parsed = parse_authors(authors);
        if !parsed.is_empty() {
            writeln!(writer, "    authors:")?;
            for (family, given) in &parsed {
                writeln!(writer, "      - family-names: {}", yaml_quote(family))?;
                if let Some(g) = given {
                    writeln!(writer, "        given-names: {}", yaml_quote(g))?;
                }
            }
        }
    }

    if let Some(year) = doc.pub_year {
        writeln!(writer, "    year: {}", year)?;
    }

    if let Some(doi) = &doc.doi {
        writeln!(writer, "    doi: {}", yaml_quote(doi))?;
    }

    if let Some(journal) = &doc.journal {
        writeln!(writer, "    journal: {}", yaml_quote(journal))?;
    } else if let Some(conference) = &doc.conference {
        writeln!(writer, "    conference: {}", yaml_quote(conference))?;
    }

    if let Some(volume) = &doc.volume {
        writeln!(writer, "    volume: {}", yaml_quote(volume))?;
    }

    if let Some(issue) = &doc.issue {
        writeln!(writer, "    issue: {}", yaml_quote(issue))?;
    }

    if let Some(pages) = format_pages(doc) {
        writeln!(writer, "    pages: {}", yaml_quote(&pages))?;
    }

    Ok(())
}

fn guess_cff_type(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "article"
    } else if doc.conference.is_some() {
        "conference-paper"
    } else {
        "generic"
    }
}

fn parse_authors(authors: &str) -> Vec<(String, Option<String>)> {
    split_authors(authors)
        .into_iter()
        .map(|a| split_author_name(&a))
        .collect()
}

fn split_author_name(name: &str) -> (String, Option<String>) {
    let name = name.trim();
    if let Some(pos) = name.find(',') {
        let family = name[..pos].trim().to_string();
        let given = name[pos + 1..].trim().to_string();
        if given.is_empty() {
            return (family, None);
        }
        return (family, Some(given));
    }
    if let Some(pos) = name.rfind(' ') {
        let given = name[..pos].trim().to_string();
        let family = name[pos + 1..].trim().to_string();
        if family.is_empty() {
            return (name.to_string(), None);
        }
        return (family, Some(given));
    }
    (name.to_string(), None)
}

fn format_pages(doc: &Document) -> Option<String> {
    match (&doc.page_start, &doc.page_end) {
        (Some(start), Some(end)) => Some(format!("{}-{}", start, end)),
        (Some(start), None) => Some(start.clone()),
        (None, _) => None,
    }
}

fn yaml_quote(s: &str) -> String {
    let s = s.replace(['\r', '\n'], " ");
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_cff_references_journal_article() {
        let doc = Document {
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("7".to_string()),
            page_start: Some("551".to_string()),
            page_end: Some("565".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_cff_references(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("type: article"), "missing type: article: {out}");
        assert!(
            out.contains("title: \"Deep Learning\""),
            "missing title: {out}"
        );
        assert!(
            out.contains("family-names: \"Smith\""),
            "missing family-names Smith: {out}"
        );
        assert!(
            out.contains("given-names: \"John\""),
            "missing given-names John: {out}"
        );
        assert!(
            out.contains("family-names: \"Lee\""),
            "missing family-names Lee: {out}"
        );
        assert!(out.contains("year: 2023"), "missing year: {out}");
        assert!(
            out.contains("doi: \"10.1234/test\""),
            "missing doi: {out}"
        );
        assert!(
            out.contains("journal: \"Nature\""),
            "missing journal: {out}"
        );
        assert!(out.contains("volume: \"42\""), "missing volume: {out}");
        assert!(out.contains("issue: \"7\""), "missing issue: {out}");
        assert!(
            out.contains("pages: \"551-565\""),
            "missing pages: {out}"
        );
    }

    #[test]
    fn test_cff_top_level_has_required_fields() {
        let doc = Document {
            title: "Test Paper".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_cff(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("cff-version: \"1.2.0\""),
            "missing cff-version: {out}"
        );
        assert!(out.contains("message:"), "missing message: {out}");
        assert!(out.contains("title:"), "missing title: {out}");
        assert!(out.contains("authors:"), "missing authors: {out}");
        assert!(out.contains("references:"), "missing references: {out}");
    }

    #[test]
    fn test_cff_conference_paper_type() {
        let doc = Document {
            title: "Conference Paper".to_string(),
            conference: Some("ICML 2023".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_cff_references(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("type: conference-paper"),
            "missing type: conference-paper: {out}"
        );
        assert!(
            out.contains("conference: \"ICML 2023\""),
            "missing conference: {out}"
        );
    }

    #[test]
    fn test_cff_references_excludes_top_level_fields() {
        let doc = Document {
            title: "Test".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_cff_references(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            !out.contains("cff-version:"),
            "should not have cff-version: {out}"
        );
        assert!(!out.contains("message:"), "should not have message: {out}");
        assert!(out.contains("references:"), "should have references: {out}");
    }

    #[test]
    fn test_cff_author_no_comma_splits_on_last_space() {
        let doc = Document {
            title: "Test".to_string(),
            authors: Some("John Smith".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_cff_references(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("family-names: \"Smith\""),
            "expected family-names Smith: {out}"
        );
        assert!(
            out.contains("given-names: \"John\""),
            "expected given-names John: {out}"
        );
    }
}
