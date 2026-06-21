use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use std::io::Write;

pub fn export_ris(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for doc in documents {
        let entry_type = guess_ris_type(doc);
        // RIS spec mandates CRLF line endings. Use `write!` with explicit `\r\n`
        // (not `writeln!`, which emits a platform-native `\n` on Unix and would
        // produce `\r\n\n` when combined with a trailing `\r\n` in the format string).
        write!(writer, "TY  - {}\r\n", entry_type)?;
        write!(writer, "TI  - {}\r\n", sanitize_ris_field(&doc.title))?;

        if let Some(authors) = &doc.authors {
            for author in split_authors(authors) {
                write!(writer, "AU  - {}\r\n", sanitize_ris_field(&author))?;
            }
        }

        if let Some(year) = doc.pub_year {
            write!(writer, "PY  - {}\r\n", year)?;
        }

        if let Some(journal) = &doc.journal {
            write!(writer, "JO  - {}\r\n", sanitize_ris_field(journal))?;
        } else if let Some(conference) = &doc.conference {
            write!(writer, "BT  - {}\r\n", sanitize_ris_field(conference))?;
        }

        if let Some(doi) = &doc.doi {
            write!(writer, "DO  - {}\r\n", sanitize_ris_field(doi))?;
        }

        if let Some(abstract_text) = &doc.abstract_text {
            write!(writer, "AB  - {}\r\n", sanitize_ris_field(abstract_text))?;
        }

        if let Some(keywords) = &doc.keywords {
            for kw in keywords.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                write!(writer, "KW  - {}\r\n", sanitize_ris_field(kw))?;
            }
        }

        if let Some(arxiv) = &doc.arxiv_id {
            write!(writer, "AN  - arXiv:{}\r\n", sanitize_ris_field(arxiv))?;
        }

        write!(writer, "ER  - \r\n")?;
    }
    Ok(())
}

/// Replace CR and LF with spaces to prevent RIS record injection.
/// A malicious title like "Foo\r\nER  - \r\n" could otherwise terminate the
/// record early and inject arbitrary follow-on records.
fn sanitize_ris_field(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

fn guess_ris_type(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "JOUR"
    } else if doc.conference.is_some() {
        "CONF"
    } else {
        "GEN"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_doc(title: &str, authors: Option<&str>) -> Document {
        Document {
            id: Some(1),
            title: title.to_string(),
            authors: authors.map(|s| s.to_string()),
            journal: None,
            conference: None,
            pub_year: None,
            doi: None,
            arxiv_id: None,
            abstract_text: None,
            keywords: None,
            file_path: None,
            file_hash: None,
            citation_key: None,
            source: None,
            rating: None,
        }
    }

    #[test]
    fn test_ris_journal_article() {
        // Given a journal article with two authors, year, journal, and DOI
        let doc = Document {
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..make_doc("Deep Learning", Some("Smith, John; Lee, Jane"))
        };
        let mut buf = Vec::new();
        // When exporting to RIS
        export_ris(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the output contains the correct RIS tags
        assert!(out.contains("TY  - JOUR"), "missing TY tag: {out}");
        assert!(out.contains("TI  - Deep Learning"), "missing TI tag: {out}");
        assert!(out.contains("AU  - Smith, John"), "missing first AU: {out}");
        assert!(out.contains("AU  - Lee, Jane"), "missing second AU: {out}");
        assert!(out.contains("PY  - 2023"), "missing PY tag: {out}");
        assert!(out.contains("JO  - Nature"), "missing JO tag: {out}");
        assert!(out.contains("DO  - 10.1234/test"), "missing DO tag: {out}");
        assert!(out.contains("ER  - "), "missing ER terminator: {out}");
    }

    #[test]
    fn test_ris_book_without_journal() {
        // Given a book (no journal, no conference)
        let doc = make_doc("Deep Learning", Some("Smith, John"));
        let mut buf = Vec::new();
        // When exporting to RIS
        export_ris(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the type is GEN and no JO/BT tags appear
        assert!(out.contains("TY  - GEN"), "expected TY-GEN for book: {out}");
        assert!(out.contains("TI  - Deep Learning"), "missing TI tag: {out}");
        assert!(out.contains("AU  - Smith, John"), "missing AU tag: {out}");
        assert!(!out.contains("JO  -"), "should not have JO for book: {out}");
        assert!(!out.contains("BT  -"), "should not have BT for book: {out}");
        assert!(out.contains("ER  - "), "missing ER terminator: {out}");
    }

    #[test]
    fn test_ris_conference_paper() {
        // Given a conference paper with conference field
        let doc = Document {
            conference: Some("ICML 2023".to_string()),
            pub_year: Some(2023),
            ..make_doc("Deep Learning", Some("Smith, John"))
        };
        let mut buf = Vec::new();
        // When exporting to RIS
        export_ris(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the type is CONF and conference goes in BT tag
        assert!(out.contains("TY  - CONF"), "expected TY-CONF: {out}");
        assert!(out.contains("TI  - Deep Learning"), "missing TI: {out}");
        assert!(out.contains("AU  - Smith, John"), "missing AU: {out}");
        assert!(out.contains("BT  - ICML 2023"), "missing BT tag: {out}");
        assert!(!out.contains("JO  -"), "should not have JO for conference: {out}");
        assert!(out.contains("ER  - "), "missing ER: {out}");
    }

    #[test]
    fn test_ris_crlf_injection_sanitized() {
        // Given a title with embedded CRLF that could inject a fake ER record
        let doc = make_doc("Evil\r\nER  - \r\nFake", Some("Smith, John"));
        let mut buf = Vec::new();
        // When exporting to RIS
        export_ris(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the CRLF is replaced with spaces — no fake ER record appears
        assert!(
            !out.contains("\r\nER  - Fake"),
            "CRLF injection succeeded — fake ER record found: {out:?}"
        );
        assert!(
            out.contains("TI  - Evil") && out.contains("Fake"),
            "expected sanitized title containing Evil and Fake: {out:?}"
        );
        // And exactly one real ER terminator exists (at start of a line)
        let er_lines = out.split("\r\n").filter(|line| line.starts_with("ER  - ")).count();
        assert_eq!(er_lines, 1, "expected exactly 1 ER line, got {er_lines}: {out:?}");
    }

    #[test]
    fn test_ris_line_endings_are_crlf() {
        // Given a journal article
        let doc = Document {
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..make_doc("Deep Learning", Some("Smith, John; Lee, Jane"))
        };
        let mut buf = Vec::new();
        // When exporting to RIS
        export_ris(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then every line ends with \r\n (no bare \n without preceding \r)
        assert!(
            !out.contains('\n') || !out.split("\r\n").any(|line| line.contains('\n')),
            "found bare \\n without preceding \\r: {out:?}"
        );
        // And no double-ending \r\n\n after ER
        assert!(
            !out.contains("\r\n\n"),
            "found double line ending \\r\\n\\n: {out:?}"
        );
    }
}
