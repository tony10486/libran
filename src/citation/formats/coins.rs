use crate::db::documents::{Document, split_authors};
use anyhow::Result;
use std::io::Write;

fn build_coins_span(doc: &Document) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push("ctx_ver=Z39.88-2004".to_string());

    let is_article = doc.journal.is_some();
    let is_conference = doc.conference.is_some();

    if is_article {
        parts.push("rft_val_fmt=info:ofi/fmt:kev:mtx:journal".to_string());
        parts.push("rft.genre=article".to_string());
    } else if is_conference {
        parts.push("rft_val_fmt=info:ofi/fmt:kev:mtx:journal".to_string());
        parts.push("rft.genre=conference".to_string());
    } else {
        parts.push("rft_val_fmt=info:ofi/fmt:kev:mtx:book".to_string());
        parts.push("rft.genre=book".to_string());
    }

    if is_article {
        parts.push(format!("rft.atitle={}", urlencoding::encode(&doc.title)));
        if let Some(j) = &doc.journal {
            parts.push(format!("rft.jtitle={}", urlencoding::encode(j)));
        }
    } else {
        parts.push(format!("rft.btitle={}", urlencoding::encode(&doc.title)));
    }

    if let Some(authors) = &doc.authors {
        let author_list = split_authors(authors);
        if let Some(first) = author_list.first() {
            if let Some(comma_pos) = first.find(',') {
                let last = first[..comma_pos].trim();
                let given = first[comma_pos + 1..].trim();
                parts.push(format!("rft.aulast={}", urlencoding::encode(last)));
                parts.push(format!("rft.aufirst={}", urlencoding::encode(given)));
            } else if let Some(space_pos) = first.rfind(' ') {
                let given = &first[..space_pos];
                let last = &first[space_pos + 1..];
                parts.push(format!("rft.aulast={}", urlencoding::encode(last)));
                parts.push(format!("rft.aufirst={}", urlencoding::encode(given)));
            } else {
                parts.push(format!("rft.au={}", urlencoding::encode(first)));
            }
        }
        for author in author_list.iter().skip(1) {
            parts.push(format!("rft.au={}", urlencoding::encode(author)));
        }
    }

    if let Some(year) = doc.pub_year {
        parts.push(format!("rft.date={}", year));
    }

    if let Some(v) = &doc.volume {
        parts.push(format!("rft.volume={}", urlencoding::encode(v)));
    }
    if let Some(i) = &doc.issue {
        parts.push(format!("rft.issue={}", urlencoding::encode(i)));
    }
    if let Some(sp) = &doc.page_start {
        parts.push(format!("rft.spage={}", urlencoding::encode(sp)));
    }
    if let Some(ep) = &doc.page_end {
        parts.push(format!("rft.epage={}", urlencoding::encode(ep)));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("rft_id=info:doi/{}", urlencoding::encode(doi)));
    } else if let Some(arxiv) = &doc.arxiv_id {
        parts.push(format!("rft_id=info:arxiv/{}", urlencoding::encode(arxiv)));
    }

    let title_attr = parts.join("&amp;");
    format!("<span class=\"Z3988\" title=\"{}\"></span>", title_attr)
}

pub fn export_coins(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for doc in documents {
        let span = build_coins_span(doc);
        writeln!(writer, "{}", span)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_coins_journal_article() {
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
        export_coins(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("class=\"Z3988\""),
            "missing Z3988 class: {out}"
        );
        assert!(
            out.contains("ctx_ver=Z39.88-2004"),
            "missing ctx_ver: {out}"
        );
        assert!(out.contains("rft.genre=article"), "missing genre: {out}");
        assert!(out.contains("rft.atitle="), "missing atitle: {out}");
        assert!(out.contains("rft_id=info:doi/"), "missing DOI: {out}");
    }

    #[test]
    fn test_coins_url_encoding() {
        let doc = Document {
            id: Some(1),
            title: "AT&T: A Review".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_coins(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.contains("AT%26T"),
            "missing URL-encoded ampersand: {out}"
        );
    }

    #[test]
    fn test_coins_book_genre() {
        let doc = Document {
            id: Some(1),
            title: "The Big Book".to_string(),
            authors: Some("Brown, Sam".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_coins(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("rft.genre=book"), "missing book genre: {out}");
        assert!(out.contains("rft.btitle="), "missing btitle: {out}");
        assert!(
            !out.contains("rft.atitle="),
            "should not have atitle for book: {out}"
        );
    }
}
