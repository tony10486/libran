//! Nature citation style template.
//!
//! Reference: Smith, J. & Brown, D. Title. J. Name 42, 123-145 (2023).
//! 6+ authors: first 1 then et al. In-text: [1]

use crate::citation::text::helpers::{format_pages, format_year, get_authors, parse_author};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_nature_authors(authors: &[String]) -> String {
    if authors.is_empty() {
        return String::new();
    }
    if authors.len() >= 6 {
        let (last, initial) = parse_author(&authors[0]);
        if initial.is_empty() {
            format!("{} et al.", last)
        } else {
            format!("{}, {} et al.", last, initial)
        }
    } else {
        let formatted: Vec<String> = authors
            .iter()
            .map(|name| {
                let (last, initial) = parse_author(name);
                if initial.is_empty() {
                    last
                } else {
                    format!("{}, {}.", last, initial)
                }
            })
            .collect();
        let (last_author, rest) = formatted.split_last().unwrap();
        if rest.is_empty() {
            last_author.clone()
        } else {
            format!("{} & {}", rest.join(", "), last_author)
        }
    }
}

pub fn render_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_nature_authors(&authors);
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("{}.", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(journal.clone());
    } else if let Some(conference) = &doc.conference {
        parts.push(conference.clone());
    }

    let year = format_year(doc.pub_year);
    let volume = doc.volume.as_deref().unwrap_or("");
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let mut vol_pages = String::new();
    if !volume.is_empty() {
        vol_pages.push_str(volume);
        if !pages.is_empty() {
            vol_pages.push_str(&format!(", {}", pages));
        }
    } else if !pages.is_empty() {
        vol_pages.push_str(&pages);
    }

    if !vol_pages.is_empty() {
        parts.push(format!("{} ({})", vol_pages, year));
    } else {
        parts.push(format!("({})", year));
    }

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
            title: "Quantum Entanglement".to_string(),
            authors: Some("Smith, John A.; Brown, David E.".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1038/nature.test".to_string()),
            volume: Some("42".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_nature_journal_article() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("Smith, J. & Brown, D."),
            "nature authors with &: {result}"
        );
        assert!(
            result.contains("Quantum Entanglement."),
            "nature title: {result}"
        );
        assert!(
            result.contains("42, 123-145 (2023)"),
            "nature vol pages year: {result}"
        );
    }

    #[test]
    fn test_nature_six_plus_authors() {
        let authors = (1..=7)
            .map(|i| format!("Author{}", i))
            .collect::<Vec<_>>()
            .join("; ");
        let doc = Document {
            title: "Many".to_string(),
            authors: Some(authors),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("et al."), "nature 6+ et al: {result}");
    }

    #[test]
    fn test_nature_in_text() {
        let doc = make_doc();
        assert_eq!(render_in_text(&doc, CitationLanguage::English), "[1]");
    }
}
