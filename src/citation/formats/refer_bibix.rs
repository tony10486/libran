use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use std::io::Write;

fn sanitize_field(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

fn doc_type(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "Journal Article"
    } else if doc.conference.is_some() {
        "Conference Paper"
    } else {
        "Generic"
    }
}

fn combined_pages(doc: &Document) -> Option<String> {
    match (&doc.page_start, &doc.page_end) {
        (Some(s), Some(e)) => Some(format!("{}-{}", s, e)),
        (Some(s), None) => Some(s.clone()),
        (None, Some(e)) => Some(e.clone()),
        (None, None) => None,
    }
}

pub fn export_refer_bibix(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for (i, doc) in documents.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }

        writeln!(writer, "%0 {}", doc_type(doc))?;

        if let Some(authors) = &doc.authors {
            for author in split_authors(authors) {
                writeln!(writer, "%A {}", sanitize_field(&author))?;
            }
        }

        writeln!(writer, "%T {}", sanitize_field(&doc.title))?;

        if let Some(j) = &doc.journal {
            writeln!(writer, "%J {}", sanitize_field(j))?;
        }
        if let Some(c) = &doc.conference {
            writeln!(writer, "%J {}", sanitize_field(c))?;
        }
        if let Some(year) = doc.pub_year {
            writeln!(writer, "%D {}", year)?;
        }
        if let Some(v) = &doc.volume {
            writeln!(writer, "%V {}", sanitize_field(v))?;
        }
        if let Some(is) = &doc.issue {
            writeln!(writer, "%N {}", sanitize_field(is))?;
        }
        if let Some(pages) = combined_pages(doc) {
            writeln!(writer, "%P {}", sanitize_field(&pages))?;
        }
        if let Some(p) = &doc.publisher {
            writeln!(writer, "%I {}", sanitize_field(p))?;
        }
        if let Some(c) = &doc.city {
            writeln!(writer, "%C {}", sanitize_field(c))?;
        }
        if let Some(doi) = &doc.doi {
            writeln!(writer, "%R {}", sanitize_field(doi))?;
        }
        if let Some(abs) = &doc.abstract_text {
            writeln!(writer, "%X {}", sanitize_field(abs))?;
        }
        if let Some(kw) = &doc.keywords {
            writeln!(writer, "%K {}", sanitize_field(kw))?;
        }
        if let Some(isbn) = &doc.isbn {
            writeln!(writer, "%@ {}", sanitize_field(isbn))?;
        } else if let Some(issn) = &doc.issn {
            writeln!(writer, "%@ {}", sanitize_field(issn))?;
        }
        if let Some(url) = &doc.url {
            writeln!(writer, "%U {}", sanitize_field(url))?;
        }
        if let Some(ad) = &doc.accessed_date {
            writeln!(writer, "%[ {}", sanitize_field(ad))?;
        }
        if let Some(ed) = &doc.edition {
            writeln!(writer, "%7 {}", sanitize_field(ed))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_refer_journal_article() {
        let doc = Document {
            id: Some(1),
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
        export_refer_bibix(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("%0 Journal Article"), "missing type: {out}");
        assert!(out.contains("%A Smith, John"), "missing first author: {out}");
        assert!(out.contains("%A Lee, Jane"), "missing second author: {out}");
        assert!(out.contains("%J Nature"), "missing journal: {out}");
        assert!(out.contains("%V 42"), "missing volume: {out}");
        assert!(out.contains("%N 7"), "missing issue: {out}");
        assert!(out.contains("%P 551-565"), "missing pages: {out}");
        assert!(out.contains("%R 10.1234/test"), "missing DOI: {out}");
    }

    #[test]
    fn test_refer_minimal_fields() {
        let doc = Document {
            id: Some(1),
            title: "Minimal Paper".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        let result = export_refer_bibix(&[doc], &mut Cursor::new(&mut buf));
        assert!(result.is_ok());
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("%T Minimal Paper"), "missing title: {out}");
        assert!(out.contains("%0 Generic"), "missing type: {out}");
    }

    #[test]
    fn test_refer_crlf_injection_prevention() {
        let doc = Document {
            id: Some(1),
            title: "Evil\r\nER  - Fake".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_refer_bibix(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            !out.lines().any(|line| line.starts_with("ER  -")),
            "ER should not appear as a separate line: {out}"
        );
        let er_count = out.lines().filter(|l| l.starts_with("ER  -")).count();
        assert_eq!(er_count, 0, "no ER line should exist: {out}");
    }
}
