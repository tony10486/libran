use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::io::Write;

const TEI_NS: &str = "http://www.tei-c.org/ns/1.0";

pub fn export_tei(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    let mut w = Writer::new_with_indent(writer, b' ', 2);

    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new("TEI");
    root.push_attribute(("xmlns", TEI_NS));
    w.write_event(Event::Start(root))?;

    w.write_event(Event::Start(BytesStart::new("teiHeader")))?;
    w.write_event(Event::Start(BytesStart::new("fileDesc")))?;
    w.write_event(Event::Start(BytesStart::new("titleStmt")))?;
    write_text_element(&mut w, "title", "Exported Bibliographic Records")?;
    w.write_event(Event::End(BytesEnd::new("titleStmt")))?;
    w.write_event(Event::Start(BytesStart::new("sourceDesc")))?;
    w.write_event(Event::Start(BytesStart::new("listBibl")))?;

    for doc in documents {
        write_bibl_struct(&mut w, doc)?;
    }

    w.write_event(Event::End(BytesEnd::new("listBibl")))?;
    w.write_event(Event::End(BytesEnd::new("sourceDesc")))?;
    w.write_event(Event::End(BytesEnd::new("fileDesc")))?;
    w.write_event(Event::End(BytesEnd::new("teiHeader")))?;
    w.write_event(Event::End(BytesEnd::new("TEI")))?;
    Ok(())
}

fn write_bibl_struct<W: Write>(w: &mut Writer<W>, doc: &Document) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("biblStruct")))?;

    let is_article = doc.journal.is_some() || doc.conference.is_some();

    if is_article {
        write_analytic(w, doc)?;
    }

    write_monogr(w, doc, is_article)?;

    w.write_event(Event::End(BytesEnd::new("biblStruct")))?;
    Ok(())
}

fn write_analytic<W: Write>(w: &mut Writer<W>, doc: &Document) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("analytic")))?;

    if let Some(authors) = &doc.authors {
        for author in split_authors(authors) {
            write_author(w, &author)?;
        }
    }

    write_titled_element(w, "title", "a", &doc.title)?;

    if let Some(doi) = &doc.doi {
        write_idno(w, "DOI", doi)?;
    }
    if let Some(arxiv) = &doc.arxiv_id {
        write_idno(w, "arXiv", arxiv)?;
    }

    w.write_event(Event::End(BytesEnd::new("analytic")))?;
    Ok(())
}

fn write_monogr<W: Write>(w: &mut Writer<W>, doc: &Document, is_article: bool) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("monogr")))?;

    if !is_article && let Some(authors) = &doc.authors {
        for author in split_authors(authors) {
            write_author(w, &author)?;
        }
    }

    if let Some(journal) = &doc.journal {
        write_titled_element(w, "title", "j", journal)?;
    } else if let Some(conference) = &doc.conference {
        write_titled_element(w, "title", "m", conference)?;
    } else {
        write_titled_element(w, "title", "m", &doc.title)?;
    }

    w.write_event(Event::Start(BytesStart::new("imprint")))?;
    if let Some(year) = doc.pub_year {
        let year_str = year.to_string();
        let mut date = BytesStart::new("date");
        date.push_attribute(("when", year_str.as_str()));
        w.write_event(Event::Start(date))?;
        w.write_event(Event::Text(BytesText::new(&year_str)))?;
        w.write_event(Event::End(BytesEnd::new("date")))?;
    }
    if let Some(publisher) = &doc.publisher {
        write_text_element(w, "publisher", publisher)?;
    }
    if let Some(city) = &doc.city {
        write_text_element(w, "pubPlace", city)?;
    }
    w.write_event(Event::End(BytesEnd::new("imprint")))?;

    if !is_article && let Some(edition) = &doc.edition {
        write_text_element(w, "edition", edition)?;
    }

    if let Some(volume) = &doc.volume {
        write_bibl_scope(w, "volume", volume)?;
    }
    if let Some(issue) = &doc.issue {
        write_bibl_scope(w, "issue", issue)?;
    }
    if let Some(start) = &doc.page_start {
        write_page_bibl_scope(w, start, doc.page_end.as_deref())?;
    }

    if let Some(isbn) = &doc.isbn {
        write_idno(w, "ISBN", isbn)?;
    }
    if let Some(issn) = &doc.issn {
        write_idno(w, "ISSN", issn)?;
    }

    w.write_event(Event::End(BytesEnd::new("monogr")))?;
    Ok(())
}

fn write_author<W: Write>(w: &mut Writer<W>, author: &str) -> Result<()> {
    w.write_event(Event::Start(BytesStart::new("author")))?;
    w.write_event(Event::Start(BytesStart::new("persName")))?;

    let (surname, forename) = parse_author_name(author);
    write_text_element(w, "surname", &surname)?;
    if !forename.is_empty() {
        write_text_element(w, "forename", &forename)?;
    }

    w.write_event(Event::End(BytesEnd::new("persName")))?;
    w.write_event(Event::End(BytesEnd::new("author")))?;
    Ok(())
}

fn parse_author_name(author: &str) -> (String, String) {
    let author = author.trim();
    if let Some(pos) = author.find(',') {
        let surname = author[..pos].trim().to_string();
        let forename = author[pos + 1..].trim().to_string();
        return (surname, forename);
    }
    if let Some(pos) = author.rfind(' ') {
        let forename = author[..pos].trim().to_string();
        let surname = author[pos + 1..].trim().to_string();
        return (surname, forename);
    }
    (author.to_string(), String::new())
}

fn write_titled_element<W: Write>(
    w: &mut Writer<W>,
    tag: &str,
    level: &str,
    text: &str,
) -> Result<()> {
    let mut elem = BytesStart::new(tag);
    elem.push_attribute(("level", level));
    w.write_event(Event::Start(elem))?;
    w.write_event(Event::Text(BytesText::new(text)))?;
    w.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

fn write_idno<W: Write>(w: &mut Writer<W>, id_type: &str, value: &str) -> Result<()> {
    let mut elem = BytesStart::new("idno");
    elem.push_attribute(("type", id_type));
    w.write_event(Event::Start(elem))?;
    w.write_event(Event::Text(BytesText::new(value)))?;
    w.write_event(Event::End(BytesEnd::new("idno")))?;
    Ok(())
}

fn write_bibl_scope<W: Write>(
    w: &mut Writer<W>,
    unit: &str,
    content: &str,
) -> Result<()> {
    let mut elem = BytesStart::new("biblScope");
    elem.push_attribute(("unit", unit));
    w.write_event(Event::Start(elem))?;
    w.write_event(Event::Text(BytesText::new(content)))?;
    w.write_event(Event::End(BytesEnd::new("biblScope")))?;
    Ok(())
}

fn write_page_bibl_scope<W: Write>(
    w: &mut Writer<W>,
    start: &str,
    end: Option<&str>,
) -> Result<()> {
    let mut elem = BytesStart::new("biblScope");
    elem.push_attribute(("unit", "page"));
    elem.push_attribute(("from", start));
    if let Some(end) = end {
        elem.push_attribute(("to", end));
    }
    w.write_event(Event::Start(elem))?;
    let content = match end {
        Some(end) => format!("{start}-{end}"),
        None => start.to_string(),
    };
    w.write_event(Event::Text(BytesText::new(&content)))?;
    w.write_event(Event::End(BytesEnd::new("biblScope")))?;
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

    #[test]
    fn test_tei_journal_article() {
        // Given a journal article with 2 authors, journal, volume, issue, pages, DOI
        let doc = Document {
            title: "Nonisothermal Diffuse Interface Model".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Siberian Journal of Industrial Mathematics".to_string()),
            pub_year: Some(2022),
            doi: Some("10.33048/SIBJIM.2022.25.103".to_string()),
            volume: Some("25".to_string()),
            issue: Some("1".to_string()),
            page_start: Some("551".to_string()),
            page_end: Some("565".to_string()),
            ..Default::default()
        };

        // When exported to TEI
        let mut buf = Vec::new();
        export_tei(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the XML contains the core TEI structural elements
        assert!(output.contains("<biblStruct>"), "missing <biblStruct>: {output}");
        assert!(output.contains("<analytic>"), "missing <analytic>: {output}");
        assert!(
            output.contains("<surname>Smith</surname>"),
            "missing surname Smith: {output}"
        );
        assert!(
            output.contains("<forename>John</forename>"),
            "missing forename John: {output}"
        );
        assert!(
            output.contains("<title level=\"a\">Nonisothermal Diffuse Interface Model</title>"),
            "missing analytic title: {output}"
        );
        assert!(output.contains("<monogr>"), "missing <monogr>: {output}");
        assert!(
            output.contains("<title level=\"j\">Siberian Journal of Industrial Mathematics</title>"),
            "missing journal title: {output}"
        );
        assert!(
            output.contains("<biblScope unit=\"volume\">25</biblScope>"),
            "missing volume biblScope: {output}"
        );
        assert!(
            output.contains("<idno type=\"DOI\">10.33048/SIBJIM.2022.25.103</idno>"),
            "missing DOI idno: {output}"
        );
        // Both authors present
        assert!(
            output.contains("<surname>Lee</surname>"),
            "missing second author surname Lee: {output}"
        );
        // Pages with from/to attributes
        assert!(
            output.contains("<biblScope unit=\"page\" from=\"551\" to=\"565\">551-565</biblScope>"),
            "missing page biblScope: {output}"
        );
    }

    #[test]
    fn test_tei_book_without_analytic() {
        // Given a book with no journal or conference
        let doc = Document {
            title: "The Rust Programming Language".to_string(),
            authors: Some("Blandy, Jim; Orendorff, Jason".to_string()),
            pub_year: Some(2023),
            publisher: Some("O'Reilly".to_string()),
            isbn: Some("978-1098139277".to_string()),
            ..Default::default()
        };

        // When exported to TEI
        let mut buf = Vec::new();
        export_tei(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then no <analytic> element is present
        assert!(
            !output.contains("<analytic>"),
            "should not have <analytic> for book: {output}"
        );
        // And authors appear in <monogr> via surname
        assert!(
            output.contains("<surname>Blandy</surname>"),
            "missing author surname Blandy in monogr: {output}"
        );
        // And title level="m"
        assert!(
            output.contains("<title level=\"m\">The Rust Programming Language</title>"),
            "missing book title level=m: {output}"
        );
        // And ISBN idno
        assert!(
            output.contains("<idno type=\"ISBN\">978-1098139277</idno>"),
            "missing ISBN idno: {output}"
        );
    }

    #[test]
    fn test_tei_xml_escaping() {
        // Given a document with special XML characters in title and journal
        let doc = Document {
            title: "A < B & C > D".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Math & Logic".to_string()),
            ..Default::default()
        };

        // When exported to TEI
        let mut buf = Vec::new();
        export_tei(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then special characters are properly escaped
        assert!(
            output.contains("A &lt; B &amp; C &gt; D"),
            "missing escaped title: {output}"
        );
        assert!(
            output.contains("Math &amp; Logic"),
            "missing escaped journal: {output}"
        );
        // And no raw unescaped special characters in content
        assert!(
            !output.contains("A < B"),
            "found unescaped < in output: {output}"
        );
    }

    #[test]
    fn test_tei_empty_input_produces_valid_skeleton() {
        // Given no documents
        // When exported to TEI
        let mut buf = Vec::new();
        export_tei(&[], &mut Cursor::new(&mut buf)).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Then the TEI skeleton is present
        assert!(
            output.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "missing XML declaration: {output}"
        );
        assert!(
            output.contains("<TEI xmlns=\"http://www.tei-c.org/ns/1.0\">"),
            "missing TEI root with namespace: {output}"
        );
        assert!(
            output.contains("<title>Exported Bibliographic Records</title>"),
            "missing header title: {output}"
        );
        assert!(
            output.contains("<listBibl>"),
            "missing listBibl open: {output}"
        );
        assert!(
            output.contains("</listBibl>"),
            "missing listBibl close: {output}"
        );
    }
}
