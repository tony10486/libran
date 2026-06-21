use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::io::Write;

/// Export a slice of `Document`s as EndNote XML to `writer`.
///
/// Produces an `<xml><records>` root containing one `<record>` element per
/// document. Only fields that are present are emitted.
pub fn export_endnote_xml(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let mut w = Writer::new_with_indent(writer, b' ', 2);

    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    w.write_event(Event::Start(BytesStart::new("xml")))?;
    w.write_event(Event::Start(BytesStart::new("records")))?;

    for doc in documents {
        write_record(&mut w, doc)?;
    }

    w.write_event(Event::End(BytesEnd::new("records")))?;
    w.write_event(Event::End(BytesEnd::new("xml")))?;
    Ok(())
}

fn write_record<W: Write>(w: &mut Writer<W>, doc: &Document) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("record")))?;

    let (ref_type_num, ref_type_name) = ref_type_for(doc);
    let mut ref_type = BytesStart::new("ref-type");
    ref_type.push_attribute(("name", ref_type_name));
    w.write_event(Event::Start(ref_type))?;
    w.write_event(Event::Text(BytesText::new(ref_type_num)))?;
    w.write_event(Event::End(BytesEnd::new("ref-type")))?;

    if let Some(authors) = &doc.authors {
        let split = split_authors(authors);
        if !split.is_empty() {
            w.write_event(Event::Start(BytesStart::new("contributors")))?;
            w.write_event(Event::Start(BytesStart::new("authors")))?;
            for author in &split {
                write_styled_text(w, "author", author)?;
            }
            w.write_event(Event::End(BytesEnd::new("authors")))?;
            w.write_event(Event::End(BytesEnd::new("contributors")))?;
        }
    }

    w.write_event(Event::Start(BytesStart::new("titles")))?;
    write_styled_text(w, "title", &doc.title)?;
    if let Some(journal) = &doc.journal {
        write_styled_text(w, "secondary-title", journal)?;
    } else if let Some(conference) = &doc.conference {
        write_styled_text(w, "secondary-title", conference)?;
    }
    w.write_event(Event::End(BytesEnd::new("titles")))?;

    if let Some(volume) = &doc.volume {
        write_styled_text(w, "volume", volume)?;
    }

    if let Some(issue) = &doc.issue {
        write_styled_text(w, "number", issue)?;
    }

    if let Some(pages) = pages_from(doc) {
        write_styled_text(w, "pages", &pages)?;
    }

    if let Some(year) = doc.pub_year {
        w.write_event(Event::Start(BytesStart::new("dates")))?;
        write_styled_text(w, "year", &year.to_string())?;
        w.write_event(Event::End(BytesEnd::new("dates")))?;
    }

    if let Some(abs_text) = &doc.abstract_text {
        write_styled_text(w, "abstract", abs_text)?;
    }

    if let Some(keywords) = &doc.keywords {
        let kws: Vec<&str> = keywords
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if !kws.is_empty() {
            w.write_event(Event::Start(BytesStart::new("keywords")))?;
            for kw in &kws {
                write_styled_text(w, "keyword", kw)?;
            }
            w.write_event(Event::End(BytesEnd::new("keywords")))?;
        }
    }

    if let Some(isbn) = &doc.isbn {
        write_styled_text(w, "isbn", isbn)?;
    } else if let Some(issn) = &doc.issn {
        write_styled_text(w, "isbn", issn)?;
    }

    if let Some(publisher) = &doc.publisher {
        write_styled_text(w, "publisher", publisher)?;
    }

    if let Some(city) = &doc.city {
        write_styled_text(w, "pub-location", city)?;
    }

    if let Some(edition) = &doc.edition {
        write_styled_text(w, "edition", edition)?;
    }

    if let Some(doi) = &doc.doi {
        write_styled_text(w, "electronic-resource-num", doi)?;
    }

    if let Some(url) = &doc.url {
        w.write_event(Event::Start(BytesStart::new("urls")))?;
        w.write_event(Event::Start(BytesStart::new("related-urls")))?;
        write_styled_text(w, "url", url)?;
        w.write_event(Event::End(BytesEnd::new("related-urls")))?;
        w.write_event(Event::End(BytesEnd::new("urls")))?;
    }

    if let Some(accessed) = &doc.accessed_date {
        write_styled_text(w, "access-date", accessed)?;
    }

    w.write_event(Event::End(BytesEnd::new("record")))?;
    Ok(())
}

fn ref_type_for(doc: &Document) -> (&'static str, &'static str) {
    if doc.journal.is_some() {
        ("17", "Journal Article")
    } else if doc.conference.is_some() {
        ("47", "Conference Paper")
    } else {
        ("13", "Generic")
    }
}

fn pages_from(doc: &Document) -> Option<String> {
    let start = doc.page_start.as_ref()?;
    Some(match &doc.page_end {
        Some(end) => format!("{start}-{end}"),
        None => start.clone(),
    })
}

fn write_styled_text<W: Write>(w: &mut Writer<W>, tag: &str, text: &str) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new(tag)))?;
    w.get_mut()
        .write_all(b"<style face=\"normal\" font=\"default\" size=\"100%\">")?;
    w.write_event(Event::Text(BytesText::new(text)))?;
    w.get_mut().write_all(b"</style>")?;
    w.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_stub_returns_ok() {
        let mut buf = Vec::new();
        let result = export_endnote_xml(&[], &mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_endnote_journal_article() {
        // Given a journal article with 2 authors, volume, issue, pages, and DOI
        let doc = Document {
            title: "Nonisothermal Diffuse Interface Model".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Siberian Journal of Industrial Mathematics".to_string()),
            pub_year: Some(2022),
            volume: Some("42".to_string()),
            issue: Some("7".to_string()),
            page_start: Some("551".to_string()),
            page_end: Some("565".to_string()),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };

        // When exported to EndNote XML
        let mut buf = Vec::new();
        export_endnote_xml(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        println!("OUTPUT:\n{output}");

        // Then the XML declares journal article ref-type 17
        assert!(
            output.contains("<ref-type name=\"Journal Article\">17</ref-type>"),
            "missing ref-type Journal Article 17: {output}"
        );
        // And each author appears in a styled <author> element
        assert!(
            output.contains("<author><style face=\"normal\" font=\"default\" size=\"100%\">Smith, John</style></author>"),
            "missing Smith, John author: {output}"
        );
        assert!(
            output.contains("<author><style face=\"normal\" font=\"default\" size=\"100%\">Lee, Jane</style></author>"),
            "missing Lee, Jane author: {output}"
        );
        // And volume, number (issue), pages, and DOI are present
        assert!(
            output.contains("<volume><style face=\"normal\" font=\"default\" size=\"100%\">42</style></volume>"),
            "missing volume: {output}"
        );
        assert!(
            output.contains("<number><style face=\"normal\" font=\"default\" size=\"100%\">7</style></number>"),
            "missing number (issue): {output}"
        );
        assert!(
            output.contains("<pages><style face=\"normal\" font=\"default\" size=\"100%\">551-565</style></pages>"),
            "missing pages: {output}"
        );
        assert!(
            output.contains("<electronic-resource-num><style face=\"normal\" font=\"default\" size=\"100%\">10.1234/test</style></electronic-resource-num>"),
            "missing electronic-resource-num (DOI): {output}"
        );
    }

    #[test]
    fn test_endnote_book_type() {
        // Given a document with no journal and no conference
        let doc = Document {
            title: "A Book Title".to_string(),
            ..Default::default()
        };

        // When exported to EndNote XML
        let mut buf = Vec::new();
        export_endnote_xml(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the ref-type is Generic 13
        assert!(
            output.contains("<ref-type name=\"Generic\">13</ref-type>"),
            "expected ref-type Generic 13, got: {output}"
        );
    }

    #[test]
    fn test_endnote_xml_escaping() {
        // Given a document whose title contains XML-special characters
        let doc = Document {
            title: "A < B & C > D".to_string(),
            ..Default::default()
        };

        // When exported to EndNote XML
        let mut buf = Vec::new();
        export_endnote_xml(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the special characters are properly escaped
        assert!(
            output.contains("A &lt; B &amp; C &gt; D"),
            "expected escaped title, got: {output}"
        );
        // And the raw characters do not appear unescaped
        assert!(
            !output.contains("A < B & C > D"),
            "unescaped characters found in output: {output}"
        );
    }
}
