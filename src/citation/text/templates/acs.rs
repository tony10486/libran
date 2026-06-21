//! ACS Guide 2022 citation style template.
//!
//! Reference format: Smith, J. A.; Lee, B. C. Title. *Journal* **Year**, *Vol*, Pages. DOI: ...
//! In-text: [1] (numeric, uses doc.id as the number)

use crate::citation::text::helpers::{
    format_pages, format_year, get_authors,
};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

pub fn render_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        if !authors.is_empty() {
            let formatted: Vec<String> = authors
                .iter()
                .map(|name| {
                    let (last, first) = crate::citation::text::helpers::parse_author_full(name);
                    if first.is_empty() {
                        return last;
                    }
                    let initials: String = first
                        .split_whitespace()
                        .filter_map(|w| w.chars().next().filter(|c| c.is_alphabetic()))
                        .map(|c| format!("{}.", c.to_uppercase().collect::<String>()))
                        .collect::<Vec<_>>()
                        .join(" ");
                    if initials.is_empty() {
                        last
                    } else {
                        format!("{}, {}", last, initials)
                    }
                })
                .collect();
            parts.push(format!("{};", formatted.join("; ")));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("{}.", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(format!("*{}*", journal));
    } else if let Some(conference) = &doc.conference {
        parts.push(format!("*{}*", conference));
    }

    let year = format_year(doc.pub_year);
    parts.push(format!("**{}**", year));

    if let Some(volume) = &doc.volume {
        let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
        if pages.is_empty() {
            parts.push(format!("*{}*", volume));
        } else {
            parts.push(format!("*{}*, {}", volume, pages));
        }
    } else {
        let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
        if !pages.is_empty() {
            parts.push(pages);
        }
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("DOI: {}", doi));
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
            title: "Catalytic Mechanism".to_string(),
            authors: Some("Smith, John A.; Lee, Bob C.".to_string()),
            journal: Some("J. Am. Chem. Soc.".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1021/jacs.test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_acs_journal_article() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Smith, J. A.; Lee, B. C.;"), "acs authors: {result}");
        assert!(result.contains("Catalytic Mechanism."), "acs title: {result}");
        assert!(result.contains("*J. Am. Chem. Soc.*"), "acs journal italic: {result}");
        assert!(result.contains("**2023**"), "acs year bold: {result}");
        assert!(result.contains("*42*, 123-145"), "acs volume+pages: {result}");
        assert!(result.contains("DOI: 10.1021/jacs.test"), "acs doi: {result}");
    }

    #[test]
    fn test_acs_in_text_numeric() {
        let doc = make_doc();
        let result = render_in_text(&doc, CitationLanguage::English);
        assert_eq!(result, "[1]");
    }

    #[test]
    fn test_acs_missing_fields() {
        let doc = Document {
            title: "Minimal".to_string(),
            authors: Some("Einstein, Albert".to_string()),
            ..Default::default()
        };
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Einstein, A.;"), "acs single author: {result}");
        assert!(result.contains("Minimal."), "acs title: {result}");
        assert!(!result.contains("DOI:"), "acs no doi: {result}");
    }
}
