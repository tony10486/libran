//! MLA 9th edition in-text citation style template.
//!
//! Reference: Smith, John A., Bob C. Jones, and David E. Brown. "Title." Journal, vol. 42, no. 3, 2023, pp. 123-45.
//! 3+ authors: first 1 then et al. In-text: (Smith, Jones, and Brown 130) — NO YEAR

use crate::citation::text::helpers::{
    format_pages, get_authors, parse_author_full,
};
use crate::citation::text::locale::{term, Term};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_mla_authors(authors: &[String], lang: CitationLanguage) -> String {
    if authors.is_empty() {
        return String::new();
    }
    let formatted: Vec<String> = authors
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let (last, first) = parse_author_full(name);
            if first.is_empty() {
                return last;
            }
            if i == 0 {
                format!("{}, {}", last, first)
            } else {
                format!("{} {}", first, last)
            }
        })
        .collect();

    if authors.len() >= 3 {
        format!("{} {}", formatted[0], term(Term::EtAl, lang))
    } else if formatted.len() == 1 {
        formatted[0].clone()
    } else {
        let (last, rest) = formatted.split_last().unwrap();
        format!("{} and {}", rest.join(", "), last)
    }
}

pub fn render_reference(doc: &Document, lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_mla_authors(&authors, lang);
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("\"{}.\"", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(format!("{},", journal));
    } else if let Some(conference) = &doc.conference {
        parts.push(format!("{},", conference));
    }

    let volume = doc.volume.as_deref();
    let issue = doc.issue.as_deref();
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let mut vol_parts: Vec<String> = Vec::new();
    if let Some(v) = volume {
        vol_parts.push(format!("vol. {}", v));
    }
    if let Some(i) = issue {
        vol_parts.push(format!("no. {}", i));
    }
    if let Some(y) = doc.pub_year {
        vol_parts.push(y.to_string());
    }
    if !pages.is_empty() {
        vol_parts.push(format!("pp. {}", pages));
    }

    if !vol_parts.is_empty() {
        parts.push(vol_parts.join(", ") + ".");
    }

    parts.join(" ")
}

pub fn render_in_text(doc: &Document, lang: CitationLanguage) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let last: Vec<String> = authors.iter().map(|n| parse_author_full(n).0).collect();
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let author_part = if last.is_empty() {
        String::new()
    } else if last.len() >= 3 {
        format!("{} {}", last[0], term(Term::EtAl, lang))
    } else if last.len() == 1 {
        last[0].clone()
    } else {
        let (l, rest) = last.split_last().unwrap();
        format!("{} and {}", rest.join(", "), l)
    };

    if pages.is_empty() {
        if author_part.is_empty() {
            String::new()
        } else {
            format!("({})", author_part)
        }
    } else {
        format!("({} {})", author_part, pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Literary Analysis".to_string(),
            authors: Some("Smith, John A.; Jones, Bob C.; Brown, David E.".to_string()),
            journal: Some("Modern Literature".to_string()),
            pub_year: Some(2023),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_mla_three_plus_et_al_reference() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("et al."), "mla 3+ et al in reference: {result}");
        assert!(result.contains("\"Literary Analysis.\""), "mla title in quotes: {result}");
        assert!(result.contains("vol. 42"), "mla vol: {result}");
        assert!(result.contains("no. 3"), "mla no: {result}");
        assert!(result.contains("pp. 123-145"), "mla pp: {result}");
    }

    #[test]
    fn test_mla_two_authors_full() {
        let doc = Document {
            title: "Test".to_string(),
            authors: Some("Smith, John A.; Jones, Bob C.".to_string()),
            journal: Some("Journal".to_string()),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Smith, John A. and Bob C. Jones"), "mla 2 authors: {result}");
        assert!(!result.contains("et al."), "mla 2 authors no et al: {result}");
    }

    #[test]
    fn test_mla_in_text_no_year() {
        let doc = make_doc();
        let result = render_in_text(&doc, CitationLanguage::English);
        assert!(result.contains("et al."), "mla in-text et al: {result}");
        assert!(!result.contains("2023"), "mla in-text NO YEAR: {result}");
        assert!(result.contains("123-145"), "mla in-text page: {result}");
    }
}
