//! IEEE v11.29.2023 citation style template.
//!
//! Reference: J. A. Smith and B. C. Lee, "Title," J. Name, vol. 42, no. 3, pp. 123-145, 2023.
//! 7+ authors: first 1 then et al. In-text: [1]

use crate::citation::text::helpers::{format_pages, format_year, get_authors};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_ieee_authors(authors: &[String]) -> String {
    if authors.is_empty() {
        return String::new();
    }
    if authors.len() >= 7 {
        let (last, first) = crate::citation::text::helpers::parse_author_full(&authors[0], None);
        let initials: String = first
            .split_whitespace()
            .filter_map(|w| w.chars().next().filter(|c| c.is_alphabetic() && !crate::citation::text::helpers::is_cjk_char(*c)))
            .map(|c| format!("{}.", c.to_uppercase().collect::<String>()))
            .collect::<Vec<_>>()
            .join(" ");
        if initials.is_empty() {
            format!("{} et al.", last)
        } else {
            format!("{} {} et al.", initials, last)
        }
    } else {
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
                    .map(|c| format!("{}.", c.to_uppercase().collect::<String>()))
                    .collect::<Vec<_>>()
                    .join(" ");
                if initials.is_empty() {
                    last
                } else {
                    format!("{} {}", initials, last)
                }
            })
            .collect();
        let (last_author, rest) = formatted.split_last().unwrap();
        if rest.is_empty() {
            last_author.clone()
        } else {
            format!("{} and {}", rest.join(", "), last_author)
        }
    }
}

pub fn render_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_ieee_authors(&authors);
        if !formatted.is_empty() {
            parts.push(format!("{},", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("\"{}\",", doc.title));
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
    if !pages.is_empty() {
        vol_parts.push(format!("pp. {}", pages));
    }

    let year = format_year(doc.pub_year);
    vol_parts.push(year.clone());

    parts.push(vol_parts.join(", ") + ".");

    if let Some(doi) = &doc.doi {
        parts.push(format!("doi: {}.", doi));
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
            title: "Neural Network Design".to_string(),
            authors: Some("Smith, John A.; Lee, Bob C.".to_string()),
            journal: Some("IEEE Trans. Pattern Anal.".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1109/TPAMI.test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_ieee_journal_article() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("J. A. Smith and B. C. Lee,"),
            "ieee authors initials first: {result}"
        );
        assert!(
            result.contains("\"Neural Network Design\","),
            "ieee title in quotes: {result}"
        );
        assert!(result.contains("vol. 42"), "ieee vol: {result}");
        assert!(result.contains("no. 3"), "ieee no: {result}");
        assert!(result.contains("pp. 123-145"), "ieee pp: {result}");
        assert!(result.contains("2023."), "ieee year: {result}");
    }

    #[test]
    fn test_ieee_seven_plus_authors() {
        let authors = (1..=8)
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
        assert!(result.contains("et al."), "ieee 7+ et al: {result}");
    }

    #[test]
    fn test_ieee_in_text() {
        let doc = make_doc();
        assert_eq!(render_in_text(&doc, CitationLanguage::English), "[1]");
    }
}
