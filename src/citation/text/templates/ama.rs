//! AMA 11th edition citation style template.
//!
//! Reference: Smith JA, Lee BC. Title. Journal. 2023;42(3):123-145. doi: 10.1234/test
//! 7+ authors: first 3 then et al.
//! In-text: [1]

use crate::citation::text::helpers::{format_pages, format_year, get_authors};
use crate::citation::text::locale::{Term, term};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_ama_authors(authors: &[String], lang: CitationLanguage) -> String {
    let etal = term(Term::EtAl, lang);
    if authors.is_empty() {
        return String::new();
    }
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = crate::citation::text::helpers::parse_author_full(name, None);
            if first.is_empty() {
                return last;
            }
            let initials: String = first
                .split_whitespace()
                .filter_map(|w| w.chars().next().filter(|c| c.is_alphabetic() && !crate::citation::text::helpers::is_cjk_char(*c)))
                .map(|c| c.to_uppercase().collect::<String>())
                .collect::<Vec<_>>()
                .join("");
            if initials.is_empty() {
                last
            } else {
                format!("{} {}", last, initials)
            }
        })
        .collect();

    if authors.len() >= 7 {
        format!("{} {}.", formatted[..3].join(", "), etal)
    } else {
        formatted.join(", ")
    }
}

pub fn render_reference(doc: &Document, lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_ama_authors(&authors, lang);
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("{}.", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(format!("{}.", journal));
    } else if let Some(conference) = &doc.conference {
        parts.push(format!("{}.", conference));
    }

    let year = format_year(doc.pub_year);
    let volume = doc.volume.as_deref().unwrap_or("");
    let issue = doc.issue.as_deref();
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let mut vol_part = year;
    if !volume.is_empty() {
        vol_part.push_str(&format!(";{}", volume));
        if let Some(is) = issue {
            vol_part.push_str(&format!("({})", is));
        }
        if !pages.is_empty() {
            vol_part.push_str(&format!(":{}", pages));
        }
    }
    parts.push(format!("{}.", vol_part));

    if let Some(doi) = &doc.doi {
        parts.push(format!("doi: {}", doi));
    }

    parts.join(" ")
}

pub fn render_in_text(doc: &Document, _lang: CitationLanguage) -> String {
    let num = doc.id.unwrap_or(1) as usize;
    format!("[{}]", num)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Clinical Trial Results".to_string(),
            authors: Some("Smith, John A.; Lee, Bob C.".to_string()),
            journal: Some("JAMA".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1001/jama.test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_ama_journal_article() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("Smith JA, Lee BC."),
            "ama authors no periods: {result}"
        );
        assert!(
            result.contains("Clinical Trial Results."),
            "ama title: {result}"
        );
        assert!(result.contains("JAMA."), "ama journal: {result}");
        assert!(
            result.contains("2023;42(3):123-145."),
            "ama year;vol(issue):pages: {result}"
        );
        assert!(
            result.contains("doi: 10.1001/jama.test"),
            "ama doi lowercase: {result}"
        );
    }

    #[test]
    fn test_ama_in_text_numeric() {
        let doc = make_doc();
        let result = render_in_text(&doc, CitationLanguage::English);
        assert_eq!(result, "[1]");
    }

    #[test]
    fn test_ama_seven_plus_authors_et_al() {
        let authors = (1..=8)
            .map(|i| format!("Author{}", i))
            .collect::<Vec<_>>()
            .join("; ");
        let doc = Document {
            title: "Many Authors".to_string(),
            authors: Some(authors),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("et al."), "ama 7+ authors et al: {result}");
    }
}
