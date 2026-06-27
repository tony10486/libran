//! MHRA 4th edition notes citation style template.
//!
//! Note: 1. John A. Smith and Bob C. Lee, 'Title,' Journal, 42.3 (2023), 123-45.
//! Bibliography: Smith, John A., and Bob C. Lee. 'Title.' Journal, 42.3 (2023), 123-45.
//! 4+ authors: "and others" (NOT et al.). In-text: [1]

use crate::citation::text::helpers::{format_pages, get_authors, parse_author_full};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_note_authors(authors: &[String]) -> String {
    if authors.is_empty() {
        return String::new();
    }
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name);
            if first.is_empty() {
                last
            } else {
                format!("{} {}", first, last)
            }
        })
        .collect();

    if formatted.len() >= 4 {
        format!("{} and others", formatted[0])
    } else if formatted.len() == 1 {
        formatted[0].clone()
    } else {
        let (last, rest) = formatted.split_last().unwrap();
        format!("{} and {}", rest.join(" and "), last)
    }
}

fn format_bib_authors(authors: &[String]) -> String {
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

    if formatted.len() >= 4 {
        format!("{}, and others", formatted[0])
    } else if formatted.len() == 1 {
        formatted[0].clone()
    } else {
        let (last, rest) = formatted.split_last().unwrap();
        format!("{}, and {}", rest.join(", "), last)
    }
}

pub fn render_reference(doc: &Document, _lang: CitationLanguage, mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Footnotes | DisplayMode::Endnotes => render_note(doc, mode),
        DisplayMode::InText => render_bibliography(doc),
    }
}

fn render_note(doc: &Document, mode: DisplayMode) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let author_str = format_note_authors(&authors);

    let title = if !doc.title.is_empty() {
        format!("'{}'", doc.title)
    } else {
        String::new()
    };

    let journal = doc
        .journal
        .clone()
        .or(doc.conference.clone())
        .unwrap_or_default();

    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    // MHRA 4th edition: Arabic numerals, volume.issue format
    let vol_issue = match (&doc.volume, &doc.issue) {
        (Some(v), Some(i)) => Some(format!("{}.{}", v, i)),
        (Some(v), None) => Some(v.clone()),
        (None, Some(i)) => Some(i.clone()),
        (None, None) => None,
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let mut elements: Vec<String> = Vec::new();
    if !author_str.is_empty() {
        elements.push(author_str);
    }
    if !title.is_empty() {
        elements.push(title);
    }
    if !journal.is_empty() {
        elements.push(journal);
    }
    let mut vol_year = String::new();
    if let Some(vi) = vol_issue {
        vol_year.push_str(&format!("{}, ", vi));
    }
    vol_year.push_str(&format!("({})", year));
    if !pages.is_empty() {
        vol_year.push_str(&format!(", {}", pages));
    }
    elements.push(vol_year);

    let body = elements.join(", ");

    match mode {
        DisplayMode::Footnotes => {
            let num = doc.id.unwrap_or(1) as usize;
            format!("{}. {}", num, body)
        }
        DisplayMode::Endnotes => {
            let num = doc.id.unwrap_or(1) as usize;
            format!("[{}] {}", num, body)
        }
        DisplayMode::InText => body,
    }
}

fn render_bibliography(doc: &Document) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_bib_authors(&authors);
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("'{}.'", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(journal.clone());
    } else if let Some(conference) = &doc.conference {
        parts.push(conference.clone());
    }

    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    // MHRA 4th edition: Arabic numerals, volume.issue format
    let vol_issue = match (&doc.volume, &doc.issue) {
        (Some(v), Some(i)) => Some(format!("{}.{}", v, i)),
        (Some(v), None) => Some(v.clone()),
        (None, Some(i)) => Some(i.clone()),
        (None, None) => None,
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let mut vol_year = String::new();
    if let Some(vi) = vol_issue {
        vol_year.push_str(&format!("{}, ", vi));
    }
    vol_year.push_str(&format!("({})", year));
    if !pages.is_empty() {
        vol_year.push_str(&format!(", {}", pages));
    }
    parts.push(format!("{}.", vol_year));

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
            title: "Literary Criticism".to_string(),
            authors: Some("Smith, John A.; Lee, Bob C.".to_string()),
            journal: Some("Modern Humanities".to_string()),
            pub_year: Some(2023),
            volume: Some("42".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_mhra_footnote() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::Footnotes);
        assert!(result.starts_with("1. "), "mhra footnote number: {result}");
        assert!(
            result.contains("John A. Smith"),
            "mhra first-name-first in note: {result}"
        );
        assert!(
            result.contains("'Literary Criticism'"),
            "mhra single-quote title: {result}"
        );
        assert!(
            result.contains("42"),
            "mhra arabic numeral volume: {result}"
        );
    }

    #[test]
    fn test_mhra_bibliography() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("Smith, John A."),
            "mhra bib last-name-first: {result}"
        );
    }

    #[test]
    fn test_mhra_four_plus_and_others() {
        let authors = (1..=5)
            .map(|i| format!("Author{}", i))
            .collect::<Vec<_>>()
            .join("; ");
        let doc = Document {
            title: "Many".to_string(),
            authors: Some(authors),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::Footnotes);
        assert!(
            result.contains("and others"),
            "mhra 'and others' not et al: {result}"
        );
        assert!(!result.contains("et al."), "mhra no et al: {result}");
    }

    #[test]
    fn test_mhra_in_text() {
        let doc = make_doc();
        assert_eq!(render_in_text(&doc, CitationLanguage::English), "[1]");
    }
}
