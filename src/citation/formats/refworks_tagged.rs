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

fn format_author(author: &str) -> String {
    if let Some(comma_pos) = author.find(',') {
        let last = author[..comma_pos].trim();
        let given = author[comma_pos + 1..].trim();
        format!("{},{}", last, given)
    } else {
        author.to_string()
    }
}

pub fn export_refworks_tagged(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for (i, doc) in documents.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }

        writeln!(writer, "RT {}", doc_type(doc))?;

        if let Some(authors) = &doc.authors {
            for author in split_authors(authors) {
                writeln!(writer, "A1 {}", sanitize_field(&format_author(&author)))?;
            }
        }

        writeln!(writer, "T1 {}", sanitize_field(&doc.title))?;

        if let Some(j) = &doc.journal {
            writeln!(writer, "JF {}", sanitize_field(j))?;
        }
        if let Some(c) = &doc.conference {
            writeln!(writer, "JF {}", sanitize_field(c))?;
        }
        if let Some(year) = doc.pub_year {
            writeln!(writer, "YR {}", year)?;
        }
        if let Some(v) = &doc.volume {
            writeln!(writer, "VO {}", sanitize_field(v))?;
        }
        if let Some(is) = &doc.issue {
            writeln!(writer, "IS {}", sanitize_field(is))?;
        }
        if let Some(sp) = &doc.page_start {
            writeln!(writer, "SP {}", sanitize_field(sp))?;
        }
        if let Some(ep) = &doc.page_end {
            writeln!(writer, "OP {}", sanitize_field(ep))?;
        }
        if let Some(kw) = &doc.keywords {
            for keyword in kw.split(',') {
                let keyword = keyword.trim();
                if !keyword.is_empty() {
                    writeln!(writer, "K1 {}", sanitize_field(keyword))?;
                }
            }
        }
        if let Some(abs) = &doc.abstract_text {
            writeln!(writer, "AB {}", sanitize_field(abs))?;
        }
        if let Some(doi) = &doc.doi {
            writeln!(writer, "DO {}", sanitize_field(doi))?;
        }
        if let Some(isbn) = &doc.isbn {
            writeln!(writer, "SN {}", sanitize_field(isbn))?;
        } else if let Some(issn) = &doc.issn {
            writeln!(writer, "SN {}", sanitize_field(issn))?;
        }
        if let Some(p) = &doc.publisher {
            writeln!(writer, "PB {}", sanitize_field(p))?;
        }
        if let Some(c) = &doc.city {
            writeln!(writer, "PP {}", sanitize_field(c))?;
        }
        if let Some(ed) = &doc.edition {
            writeln!(writer, "ED {}", sanitize_field(ed))?;
        }
        if let Some(url) = &doc.url {
            writeln!(writer, "LK {}", sanitize_field(url))?;
        }
        if let Some(ad) = &doc.accessed_date {
            writeln!(writer, "RD {}", sanitize_field(ad))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_refworks_journal_article() {
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
        export_refworks_tagged(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("RT Journal Article"), "missing RT: {out}");
        assert!(out.contains("A1 Smith,John"), "missing first author: {out}");
        assert!(out.contains("A1 Lee,Jane"), "missing second author: {out}");
        assert!(out.contains("JF Nature"), "missing journal: {out}");
        assert!(out.contains("VO 42"), "missing volume: {out}");
        assert!(out.contains("IS 7"), "missing issue: {out}");
        assert!(out.contains("SP 551"), "missing start page: {out}");
        assert!(out.contains("OP 565"), "missing end page: {out}");
        assert!(out.contains("DO 10.1234/test"), "missing DOI: {out}");
    }

    #[test]
    fn test_refworks_keywords_repeated() {
        let doc = Document {
            id: Some(1),
            title: "ML Paper".to_string(),
            keywords: Some("AI, ML".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_refworks_tagged(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("K1 AI"), "missing first keyword: {out}");
        assert!(out.contains("K1 ML"), "missing second keyword: {out}");
    }

    #[test]
    fn test_refworks_crlf_injection_prevention() {
        let doc = Document {
            id: Some(1),
            title: "Evil\r\nRT Fake".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_refworks_tagged(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let rt_count = out.lines().filter(|l| l.starts_with("RT Fake")).count();
        assert_eq!(rt_count, 0, "no RT Fake line should exist: {out}");
    }
}
