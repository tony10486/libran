use crate::citation::text::{
    render_citation, CitationLanguage, CitationStyle, DisplayMode,
};
use crate::db::documents::{split_authors, Document};
use anyhow::Result;
use std::io::Write;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn doc_url(doc: &Document) -> Option<String> {
    if let Some(doi) = &doc.doi {
        Some(format!("https://doi.org/{}", doi))
    } else if let Some(arxiv) = &doc.arxiv_id {
        Some(format!("https://arxiv.org/abs/{}", arxiv))
    } else if let Some(url) = &doc.url {
        Some(url.clone())
    } else {
        None
    }
}

pub fn export_bookmarks(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    writeln!(writer, "<!DOCTYPE NETSCAPE-Bookmark-file-1>")?;
    writeln!(
        writer,
        "<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">"
    )?;
    writeln!(writer, "<TITLE>Bookmarks</TITLE>")?;
    writeln!(writer, "<H1>Bookmarks</H1>")?;
    writeln!(writer, "<DL><p>")?;

    for doc in documents {
        let url = match doc_url(doc) {
            Some(u) => u,
            None => continue,
        };

        let citation_text = render_citation(
            doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap_or_else(|_| doc.title.clone());

        let escaped_text = html_escape(&citation_text);
        let escaped_url = html_escape(&url);

        let add_date = doc
            .pub_year
            .map(|y| format!("{}-01-01T00:00:00Z", y))
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        writeln!(
            writer,
            "    <DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>",
            escaped_url, add_date, escaped_text
        )?;

        if let Some(authors) = &doc.authors {
            let author_list = split_authors(authors);
            if !author_list.is_empty() {
                let names = author_list
                    .iter()
                    .map(|a| html_escape(a))
                    .collect::<Vec<_>>()
                    .join("; ");
                writeln!(writer, "    <DD>{}", names)?;
            }
        }
    }

    writeln!(writer, "</DL><p>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bookmarks_journal_article_with_doi() {
        let doc = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bookmarks(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"), "missing DOCTYPE: {out}");
        assert!(
            out.contains("HREF=\"https://doi.org/10.1234/test\""),
            "missing DOI URL: {out}"
        );
        assert!(out.contains("Smith, J."), "missing citation text: {out}");
    }

    #[test]
    fn test_bookmarks_arxiv_url_fallback() {
        let doc = Document {
            id: Some(1),
            title: "Quantum Computing".to_string(),
            authors: Some("Lee, Jane".to_string()),
            arxiv_id: Some("2301.12345".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bookmarks(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("https://arxiv.org/abs/2301.12345"),
            "missing arXiv URL: {out}"
        );
    }

    #[test]
    fn test_bookmarks_skips_doc_without_url() {
        let doc = Document {
            id: Some(1),
            title: "No URL Paper".to_string(),
            authors: Some("Brown, Sam".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bookmarks(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains("No URL Paper"), "should skip doc without URL: {out}");
    }
}
