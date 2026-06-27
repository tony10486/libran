use crate::db::documents::{Document, split_authors};
use anyhow::Result;
use std::io::Write;

fn doc_qid(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "Q13442814"
    } else if doc.conference.is_some() {
        "Q23927052"
    } else {
        "Q386724"
    }
}

fn qs_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn export_wikidata_qs(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for (i, doc) in documents.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }

        writeln!(writer, "CREATE")?;
        writeln!(writer, "LAST\tP31\t{}", doc_qid(doc))?;
        writeln!(writer, "LAST\tLen\t\"{}\"", qs_escape(&doc.title))?;
        writeln!(writer, "LAST\tP1476\ten:\"{}\"", qs_escape(&doc.title))?;

        if let Some(authors) = &doc.authors {
            for (idx, author) in split_authors(authors).iter().enumerate() {
                let ordinal = idx + 1;
                writeln!(
                    writer,
                    "LAST\tP2093\t\"{}\"\tP1545\t\"{}\"",
                    qs_escape(author),
                    ordinal
                )?;
            }
        }

        if let Some(year) = doc.pub_year {
            writeln!(writer, "LAST\tP577\t+{}-01-01T00:00:00Z/9", year)?;
        }

        if let Some(doi) = &doc.doi {
            writeln!(writer, "LAST\tP356\t\"{}\"", qs_escape(&doi.to_uppercase()))?;
        }

        if let Some(arxiv) = &doc.arxiv_id {
            writeln!(writer, "LAST\tP818\t\"{}\"", qs_escape(arxiv))?;
        }

        if let Some(journal) = &doc.journal {
            writeln!(writer, "LAST\tP1433\ten:\"{}\"", qs_escape(journal))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_qs_journal_article() {
        let doc = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_wikidata_qs(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("CREATE"), "missing CREATE: {out}");
        assert!(out.contains("LAST\tP31\tQ13442814"), "missing P31: {out}");
        assert!(
            out.contains("P1476\ten:\"Deep Learning\""),
            "missing P1476: {out}"
        );
        assert!(
            out.contains("P2093\t\"Smith, John\"\tP1545\t\"1\""),
            "missing first author: {out}"
        );
        assert!(
            out.contains("P2093\t\"Lee, Jane\"\tP1545\t\"2\""),
            "missing second author: {out}"
        );
        assert!(
            out.contains("P577\t+2023-01-01T00:00:00Z/9"),
            "missing P577: {out}"
        );
        assert!(
            out.contains("P356\t\"10.1234/TEST\""),
            "missing uppercase DOI: {out}"
        );
    }

    #[test]
    fn test_qs_arxiv_id() {
        let doc = Document {
            id: Some(1),
            title: "Quantum Paper".to_string(),
            arxiv_id: Some("2301.12345".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_wikidata_qs(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("P818\t\"2301.12345\""),
            "missing arXiv P818: {out}"
        );
    }

    #[test]
    fn test_qs_multiple_documents_separated() {
        let doc1 = Document {
            id: Some(1),
            title: "Paper One".to_string(),
            ..Default::default()
        };
        let doc2 = Document {
            id: Some(2),
            title: "Paper Two".to_string(),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_wikidata_qs(&[doc1, doc2], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let create_count = out.matches("CREATE").count();
        assert_eq!(create_count, 2, "should have 2 CREATE statements: {out}");
        assert!(out.contains("Paper One"), "missing first title: {out}");
        assert!(out.contains("Paper Two"), "missing second title: {out}");
    }
}
