//! NLM/Vancouver Citing Medicine 2nd edition citation style template.
//!
//! Reference: Smith JA, Lee BC. Title. J Name. 2023;42(3):123-45.
//! 7+ authors: first 6 then et al. In-text: (1)

use crate::citation::text::helpers::{format_pages, format_year, get_authors};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_vancouver_authors(authors: &[String]) -> String {
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
                .filter_map(|w| {
                    w.chars().next().filter(|c| {
                        c.is_alphabetic() && !crate::citation::text::helpers::is_cjk_char(*c)
                    })
                })
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
        format!("{} et al.", formatted[..6].join(", "))
    } else {
        formatted.join(", ")
    }
}

fn shorten_pages(pages: &str) -> String {
    if let Some(dash_pos) = pages.find('-') {
        let start = &pages[..dash_pos];
        let end = &pages[dash_pos + 1..];
        if end.len() > 2 && end.len() <= start.len() {
            let common_len = start.len() - end.len() + 2;
            if common_len <= start.len() {
                let shortened = &start[common_len..];
                if end.ends_with(shortened) {
                    return format!(
                        "{}-{}",
                        start,
                        &end[end.len() - (end.len() - shortened_len(end, start))..]
                    );
                }
            }
        }
    }
    pages.to_string()
}

fn shortened_len(end: &str, start: &str) -> usize {
    let min = end.len().min(start.len());
    for i in (0..min).rev() {
        if end.ends_with(&start[start.len() - i..]) {
            return i;
        }
    }
    end.len()
}

pub fn render_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_vancouver_authors(&authors);
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
    let short_pages = shorten_pages(&pages);

    let mut vol_part = year;
    if !volume.is_empty() {
        vol_part.push_str(&format!(";{}", volume));
        if let Some(is) = issue {
            vol_part.push_str(&format!("({})", is));
        }
        if !short_pages.is_empty() {
            vol_part.push_str(&format!(":{}", short_pages));
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
    format!("({})", num)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Systematic Review".to_string(),
            authors: Some("Smith, John A.; Lee, Bob C.".to_string()),
            journal: Some("BMJ".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1136/bmj.test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_vancouver_journal_article() {
        let doc = make_doc();
        let result = render_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("Smith JA, Lee BC."),
            "vancouver authors no periods: {result}"
        );
        assert!(
            result.contains("Systematic Review."),
            "vancouver title: {result}"
        );
        assert!(result.contains("BMJ."), "vancouver journal: {result}");
        assert!(
            result.contains("2023;42(3):"),
            "vancouver year;vol(issue): {result}"
        );
    }

    #[test]
    fn test_vancouver_seven_plus_authors() {
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
        assert!(result.contains("et al."), "vancouver 7+ et al: {result}");
    }

    #[test]
    fn test_vancouver_in_text_parens() {
        let doc = make_doc();
        assert_eq!(render_in_text(&doc, CitationLanguage::English), "(1)");
    }
}
