use crate::citation::text::{CitationLanguage, CitationStyle, DisplayMode, render_citation};
use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

pub fn export_evernote(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    write!(
        writer,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <!DOCTYPE en-export SYSTEM \"http://xml.evernote.com/pub/evernote-export.dtd\">\
         <en-export export-date=\"20230101T000000Z\" application=\"Libran\" version=\"1.0\">"
    )?;

    for doc in documents {
        write_note(writer, doc)?;
    }

    write!(writer, "</en-export>")?;
    Ok(())
}

fn write_note(writer: &mut impl Write, doc: &Document) -> Result<()> {
    write!(writer, "<note>")?;

    write!(writer, "<title>{}</title>", html_escape(&doc.title))?;

    let citation_text = render_citation(
        doc,
        CitationStyle::Apa7th,
        CitationLanguage::English,
        DisplayMode::InText,
    )
    .unwrap_or_else(|_| doc.title.clone());
    let content = build_content(&citation_text, doc);
    write!(writer, "<content><![CDATA[{content}]]></content>")?;

    let timestamp = if doc.pub_year.is_some() {
        "20230101T000000Z"
    } else {
        "19700101T000000Z"
    };
    write!(writer, "<created>{timestamp}</created>")?;
    write!(writer, "<updated>{timestamp}</updated>")?;

    if let Some(keywords) = &doc.keywords {
        for kw in keywords.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            write!(writer, "<tag>{}</tag>", html_escape(kw))?;
        }
    }

    write!(writer, "<note-attributes>")?;
    if let Some(authors) = &doc.authors {
        write!(writer, "<author>{}</author>", html_escape(authors))?;
    }
    if let Some(url) = source_url(doc) {
        write!(writer, "<source-url>{}</source-url>", html_escape(&url))?;
    }
    write!(writer, "</note-attributes>")?;

    write!(writer, "</note>")?;
    Ok(())
}

fn build_content(citation_text: &str, doc: &Document) -> String {
    let mut body = String::new();
    body.push_str(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                   <!DOCTYPE en-note SYSTEM \"http://xml.evernote.com/pub/enml.dtd\">\
                   <en-note>",
    );
    body.push_str(&format!("<div>{}</div>", citation_text));

    if let Some(abstract_text) = &doc.abstract_text {
        body.push_str(&format!(
            "<div><strong>Abstract:</strong> {}</div>",
            abstract_text
        ));
    }

    if let Some(doi) = &doc.doi {
        body.push_str(&format!(
            "<div><strong>DOI:</strong> <a href=\"https://doi.org/{doi}\">{doi}</a></div>"
        ));
    }

    body.push_str("</en-note>");
    body
}

fn source_url(doc: &Document) -> Option<String> {
    if let Some(doi) = &doc.doi {
        return Some(format!("https://doi.org/{doi}"));
    }
    if let Some(arxiv) = &doc.arxiv_id {
        return Some(format!("https://arxiv.org/abs/{arxiv}"));
    }
    None
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_doc(title: &str, authors: Option<&str>) -> Document {
        Document {
            title: title.to_string(),
            authors: authors.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_evernote_note_structure() {
        // Given a document with title, authors, pub_year, and DOI
        let doc = Document {
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..make_doc("Deep Learning", Some("Smith, John"))
        };
        let mut buf = Vec::new();
        // When exporting to ENEX
        export_evernote(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the ENEX envelope is present
        assert!(out.contains("<en-export "), "missing en-export root: {out}");
        assert!(
            out.contains("</en-export>"),
            "missing en-export footer: {out}"
        );
        // And a note element wraps the document
        assert!(out.contains("<note>"), "missing note element: {out}");
        assert!(out.contains("</note>"), "missing note close: {out}");
        // And the title is HTML-escaped into a title element
        assert!(
            out.contains("<title>Deep Learning</title>"),
            "missing title element: {out}"
        );
        // And the content is a CDATA section containing an en-note body
        assert!(
            out.contains("<content><![CDATA["),
            "missing content CDATA open: {out}"
        );
        assert!(
            out.contains("<en-note>"),
            "missing en-note in content: {out}"
        );
        // And the APA citation text appears inside the content
        assert!(
            out.contains("Smith, J. (2023). Deep Learning."),
            "missing citation text in content: {out}"
        );
        // And the source-url points to the DOI URL
        assert!(
            out.contains("<source-url>https://doi.org/10.1234/test</source-url>"),
            "missing DOI source-url: {out}"
        );
        // And created/updated timestamps use the fixed format with pub_year present
        assert!(
            out.contains("<created>20230101T000000Z</created>"),
            "missing created timestamp: {out}"
        );
        assert!(
            out.contains("<updated>20230101T000000Z</updated>"),
            "missing updated timestamp: {out}"
        );
    }

    #[test]
    fn test_evernote_tags_from_keywords() {
        // Given a document with comma-separated keywords "AI, ML"
        let doc = Document {
            keywords: Some("AI, ML".to_string()),
            ..make_doc("Tagged Paper", Some("Lee, Jane"))
        };
        let mut buf = Vec::new();
        // When exporting to ENEX
        export_evernote(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then exactly two tag elements are emitted, one per keyword
        let tag_count = out.matches("<tag>").count();
        assert_eq!(tag_count, 2, "expected 2 tags, got {tag_count}: {out}");
        assert!(
            out.contains("<tag>AI</tag>") && out.contains("<tag>ML</tag>"),
            "missing AI/ML tag elements: {out}"
        );
    }

    #[test]
    fn test_evernote_skips_url_when_no_doi_or_arxiv() {
        // Given a document with neither DOI nor arXiv id
        let doc = make_doc("No URL Paper", Some("Doe, Jane"));
        let mut buf = Vec::new();
        // When exporting to ENEX
        export_evernote(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then no source-url element is emitted
        assert!(
            !out.contains("<source-url>"),
            "expected no source-url without DOI/arXiv: {out}"
        );
        // But note-attributes with author still appears
        assert!(
            out.contains("<author>Doe, Jane</author>"),
            "missing author in note-attributes: {out}"
        );
    }

    #[test]
    fn test_evernote_html_escapes_title_and_tags() {
        // Given a title and keyword containing XML-special characters
        let doc = Document {
            keywords: Some("a&b<c>".to_string()),
            ..make_doc("A & B <C> \"D\"", Some("Smith, John"))
        };
        let mut buf = Vec::new();
        // When exporting to ENEX
        export_evernote(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the title is HTML-escaped
        assert!(
            out.contains("<title>A &amp; B &lt;C&gt; &quot;D&quot;</title>"),
            "title not escaped: {out}"
        );
        // And the tag is HTML-escaped
        assert!(
            out.contains("<tag>a&amp;b&lt;c&gt;</tag>"),
            "tag not escaped: {out}"
        );
    }

    #[test]
    fn test_evernote_arxiv_source_url() {
        // Given a document with an arXiv id but no DOI
        let doc = Document {
            arxiv_id: Some("2310.0001".to_string()),
            ..make_doc("ArXiv Paper", Some("Lee, Jane"))
        };
        let mut buf = Vec::new();
        // When exporting to ENEX
        export_evernote(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Then the source-url points to the arXiv abstract
        assert!(
            out.contains("<source-url>https://arxiv.org/abs/2310.0001</source-url>"),
            "missing arXiv source-url: {out}"
        );
    }
}
