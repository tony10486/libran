use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::io::Write;

/// Export a slice of `Document`s as MODS 3.7 XML to `writer`.
///
/// Produces a `<modsCollection version="3.7">` root containing one `<mods>`
/// element per document. Only fields that are present are emitted.
pub fn export_mods(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let mut w = Writer::new_with_indent(writer, b' ', 2);

    // XML declaration: <?xml version="1.0" encoding="UTF-8"?>
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Root: <modsCollection version="3.7">
    let mut root = BytesStart::new("modsCollection");
    root.push_attribute(("version", "3.7"));
    w.write_event(Event::Start(root))?;

    for doc in documents {
        write_mods_element(&mut w, doc)?;
    }

    // Close root
    w.write_event(Event::End(BytesEnd::new("modsCollection")))?;
    Ok(())
}

fn write_mods_element<W: Write>(w: &mut Writer<W>, doc: &Document) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("mods")))?;

    // titleInfo / title (always present — title is required on Document)
    w.write_event(Event::Start(BytesStart::new("titleInfo")))?;
    write_text_element(w, "title", &doc.title)?;
    w.write_event(Event::End(BytesEnd::new("titleInfo")))?;

    // name type="personal" / namePart — one per author
    if let Some(authors) = &doc.authors {
        for author in split_authors(authors) {
            let mut name = BytesStart::new("name");
            name.push_attribute(("type", "personal"));
            w.write_event(Event::Start(name))?;
            write_text_element(w, "namePart", &author)?;
            w.write_event(Event::End(BytesEnd::new("name")))?;
        }
    }

    // originInfo / dateCreated — if pub_year present
    if let Some(year) = doc.pub_year {
        w.write_event(Event::Start(BytesStart::new("originInfo")))?;
        write_text_element(w, "dateCreated", &year.to_string())?;
        w.write_event(Event::End(BytesEnd::new("originInfo")))?;
    }

    // relatedItem type="host" / titleInfo / title — journal or conference
    if let Some(journal) = &doc.journal {
        write_host_title(w, journal)?;
    } else if let Some(conference) = &doc.conference {
        write_host_title(w, conference)?;
    }

    // identifier type="doi"
    if let Some(doi) = &doc.doi {
        write_identifier(w, "doi", doi)?;
    }

    // identifier type="arxiv"
    if let Some(arxiv) = &doc.arxiv_id {
        write_identifier(w, "arxiv", arxiv)?;
    }

    // abstract
    if let Some(abs_text) = &doc.abstract_text {
        write_text_element(w, "abstract", abs_text)?;
    }

    // classification — keywords (comma-separated)
    if let Some(keywords) = &doc.keywords {
        for kw in keywords.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            write_text_element(w, "classification", kw)?;
        }
    }

    w.write_event(Event::End(BytesEnd::new("mods")))?;
    Ok(())
}

fn write_host_title<W: Write>(w: &mut Writer<W>, title: &str) -> Result<()> {
    let mut related = BytesStart::new("relatedItem");
    related.push_attribute(("type", "host"));
    w.write_event(Event::Start(related))?;
    w.write_event(Event::Start(BytesStart::new("titleInfo")))?;
    write_text_element(w, "title", title)?;
    w.write_event(Event::End(BytesEnd::new("titleInfo")))?;
    w.write_event(Event::End(BytesEnd::new("relatedItem")))?;
    Ok(())
}

fn write_identifier<W: Write>(w: &mut Writer<W>, id_type: &str, value: &str) -> Result<()> {
    let mut elem = BytesStart::new("identifier");
    elem.push_attribute(("type", id_type));
    w.write_event(Event::Start(elem))?;
    w.write_event(Event::Text(BytesText::new(value)))?;
    w.write_event(Event::End(BytesEnd::new("identifier")))?;
    Ok(())
}

fn write_text_element<W: Write>(w: &mut Writer<W>, tag: &str, text: &str) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new(tag)))?;
    w.write_event(Event::Text(BytesText::new(text)))?;
    w.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::documents::Document;
    use std::io::Cursor;

    #[allow(clippy::too_many_arguments)]
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
        }
    }

    #[test]
    fn test_journal_article_contains_core_mods_elements() {
        // Given a journal article with title, one author, journal, year, and DOI
        let doc = make_doc(
            Some(1),
            "Nonisothermal Diffuse Interface Model",
            Some("Smith, John"),
            Some("Siberian Journal of Industrial Mathematics"),
            None,
            Some(2022),
            Some("10.33048/SIBJIM.2022.25.103"),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // When exported to MODS
        let mut buf = Vec::new();
        export_mods(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the XML contains the core MODS structural elements
        assert!(output.contains("<mods"), "missing <mods element: {output}");
        assert!(
            output.contains("<titleInfo>"),
            "missing <titleInfo>: {output}"
        );
        assert!(
            output.contains("<title>Nonisothermal Diffuse Interface Model</title>"),
            "missing <title> with content: {output}"
        );
        assert!(
            output.contains("<name type=\"personal\">"),
            "missing <name type=\"personal\">: {output}"
        );
        assert!(
            output.contains("<namePart>Smith, John</namePart>"),
            "missing <namePart> with author: {output}"
        );
        assert!(
            output.contains("<dateCreated>2022</dateCreated>"),
            "missing <dateCreated> with year: {output}"
        );
        assert!(
            output.contains("<identifier type=\"doi\">10.33048/SIBJIM.2022.25.103</identifier>"),
            "missing <identifier type=\"doi\">: {output}"
        );
    }

    #[test]
    fn test_multiple_authors_produce_one_name_element_each() {
        // Given a document with two semicolon-separated authors
        let doc = make_doc(
            Some(2),
            "Multi-author Paper",
            Some("Smith, John; Lee, Jane"),
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

        // When exported to MODS
        let mut buf = Vec::new();
        export_mods(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then exactly two <name type="personal"> elements are present
        let name_count = output.matches("<name type=\"personal\">").count();
        assert_eq!(
            name_count, 2,
            "expected 2 <name type=\"personal\"> elements, got {name_count}: {output}"
        );

        // And each author appears in their own <namePart>
        assert!(
            output.contains("<namePart>Smith, John</namePart>"),
            "missing Smith, John namePart: {output}"
        );
        assert!(
            output.contains("<namePart>Lee, Jane</namePart>"),
            "missing Lee, Jane namePart: {output}"
        );
    }

    #[test]
    fn test_doi_and_arxiv_identifiers_both_emitted() {
        // Given a document with both a DOI and an arXiv ID
        let doc = make_doc(
            Some(3),
            "Dual Identifier Paper",
            Some("Einstein, Albert"),
            None,
            None,
            Some(1905),
            Some("10.1002/andp.19053220607"),
            Some("physics/0501001"),
            None,
            None,
            None,
            None,
            None,
        );

        // When exported to MODS
        let mut buf = Vec::new();
        export_mods(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then both identifier types are present
        assert!(
            output.contains("<identifier type=\"doi\">10.1002/andp.19053220607</identifier>"),
            "missing DOI identifier: {output}"
        );
        assert!(
            output.contains("<identifier type=\"arxiv\">physics/0501001</identifier>"),
            "missing arXiv identifier: {output}"
        );
    }
}
