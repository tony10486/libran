use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

/// Export a slice of `Document`s as RFC 4180 CSV to `writer`.
///
/// Columns (in order):
/// id, title, authors, journal, conference, pub_year, doi, arxiv_id,
/// abstract, keywords, citation_key, source, rating
///
/// `None` fields become empty cells; the csv crate handles quoting.
pub fn export_csv(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);
    wtr.write_record([
        "id",
        "title",
        "authors",
        "journal",
        "conference",
        "pub_year",
        "doi",
        "arxiv_id",
        "abstract",
        "keywords",
        "citation_key",
        "source",
        "rating",
    ])?;

    for doc in documents {
        wtr.write_record([
            doc.id.map(|i| i.to_string()).unwrap_or_default(),
            doc.title.clone(),
            doc.authors.clone().unwrap_or_default(),
            doc.journal.clone().unwrap_or_default(),
            doc.conference.clone().unwrap_or_default(),
            doc.pub_year.map(|y| y.to_string()).unwrap_or_default(),
            doc.doi.clone().unwrap_or_default(),
            doc.arxiv_id.clone().unwrap_or_default(),
            doc.abstract_text.clone().unwrap_or_default(),
            doc.keywords.clone().unwrap_or_default(),
            doc.citation_key.clone().unwrap_or_default(),
            doc.source.clone().unwrap_or_default(),
            doc.rating.map(|r| r.to_string()).unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::documents::Document;
    use std::io::Cursor;

    fn make_doc(
        id: Option<i64>,
        title: &str,
        authors: Option<&str>,
        journal: Option<&str>,
        conference: Option<&str>,
        pub_year: Option<i64>,
        doi: Option<&str>,
        arxiv_id: Option<&str>,
        abstract_text: Option<&str>,
        keywords: Option<&str>,
        citation_key: Option<&str>,
        source: Option<&str>,
        rating: Option<i64>,
    ) -> Document {
        Document {
            id,
            title: title.to_string(),
            authors: authors.map(String::from),
            journal: journal.map(String::from),
            conference: conference.map(String::from),
            pub_year,
            doi: doi.map(String::from),
            arxiv_id: arxiv_id.map(String::from),
            abstract_text: abstract_text.map(String::from),
            keywords: keywords.map(String::from),
            file_path: None,
            file_hash: None,
            citation_key: citation_key.map(String::from),
            source: source.map(String::from),
            rating,
            ..Default::default()
        }
    }

    #[test]
    fn test_journal_article_writes_header_and_one_data_row() {
        // Given a single fully-populated journal article
        let doc = make_doc(
            Some(1),
            "Test",
            Some("Smith, John; Doe, Jane"),
            Some("Nature"),
            None,
            Some(2024),
            Some("10.1038/s41586-024-00001-x"),
            None,
            Some("An abstract."),
            Some("physics, math"),
            Some("Smith2024Test"),
            Some("crossref"),
            Some(5),
        );

        // When exported to CSV
        let mut buf = Vec::new();
        export_csv(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the header row is present with the canonical column order
        let header = "id,title,authors,journal,conference,pub_year,doi,arxiv_id,abstract,keywords,citation_key,source,rating";
        assert!(
            output.starts_with(header),
            "expected output to start with header, got: {output}"
        );

        // And exactly one data row follows the header (2 lines total)
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            2,
            "expected header + 1 data row, got {} lines: {output}",
            lines.len()
        );

        // And the data row contains the mapped field values
        let data = lines[1];
        assert!(data.contains("1"), "id should be 1: {data}");
        assert!(data.contains("Test"), "title should be present: {data}");
        assert!(
            data.contains("Smith, John; Doe, Jane"),
            "authors should be present: {data}"
        );
        assert!(data.contains("Nature"), "journal should be present: {data}");
        assert!(data.contains("2024"), "pub_year should be 2024: {data}");
        assert!(
            data.contains("10.1038/s41586-024-00001-x"),
            "doi should be present: {data}"
        );
        assert!(
            data.contains("An abstract."),
            "abstract should be present: {data}"
        );
        assert!(
            data.contains("physics, math"),
            "keywords should be present: {data}"
        );
        assert!(
            data.contains("Smith2024Test"),
            "citation_key should be present: {data}"
        );
        assert!(data.contains("crossref"), "source should be present: {data}");
        assert!(data.contains("5"), "rating should be 5: {data}");
    }

    #[test]
    fn test_title_with_comma_is_rfc4180_quoted() {
        // Given a document whose title contains a comma (requires quoting)
        let doc = make_doc(
            Some(2),
            "Deep, Learning: A Survey",
            Some("LeCun, Yann"),
            None,
            None,
            Some(2015),
            None,
            Some("1503.02531"),
            None,
            None,
            None,
            None,
            None,
        );

        // When exported to CSV
        let mut buf = Vec::new();
        export_csv(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the title field is wrapped in double quotes per RFC 4180
        assert!(
            output.contains("\"Deep, Learning: A Survey\""),
            "expected title with comma to be quoted, got: {output}"
        );

        // And the literal unquoted form does NOT appear
        assert!(
            !output.contains(",Deep, Learning: A Survey,"),
            "unquoted title with comma should not appear: {output}"
        );
    }

    #[test]
    fn test_missing_fields_become_empty_cells_not_none() {
        // Given a document with many fields set to None
        let doc = make_doc(
            None,
            "Sparse",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // When exported to CSV
        let mut buf = Vec::new();
        export_csv(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the literal string "None" must NOT appear anywhere
        assert!(
            !output.contains("None"),
            "missing fields must be empty cells, not the string \"None\": {output}"
        );

        // And the data row is exactly: empty,title, then 11 empty cells
        // (13 fields total => 12 commas; id empty, title="Sparse", rest empty)
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2, "expected 2 lines: {output}");
        assert_eq!(
            lines[1], ",Sparse,,,,,,,,,,,",
            "expected all-None fields to be empty cells: {output}"
        );
    }
}
