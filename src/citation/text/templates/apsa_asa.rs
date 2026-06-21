//! APSA 2018 and ASA 6th/7th edition citation style templates.
//!
//! APSA reference: Smith, John A., Bob C. Jones, and David E. Brown. 2023. "Title." Journal 42 (3): 123-45.
//! ASA reference: Same but 42(3):123-45. (no spaces)
//! In-text APSA: (Smith, Jones, and Brown 2023, 130)
//! In-text ASA: (Smith, Jones, and Brown 2023:130) (colon instead of comma)

use crate::citation::text::helpers::{
    format_pages, get_authors, parse_author_full,
};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_full_authors_first_last(authors: &[String]) -> String {
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

    if formatted.len() == 1 {
        formatted[0].clone()
    } else {
        let (last, rest) = formatted.split_last().unwrap();
        format!("{}, and {}", rest.join(", "), last)
    }
}

fn last_names(authors: &[String]) -> Vec<String> {
    authors.iter().map(|n| parse_author_full(n).0).collect()
}

pub fn render_apsa_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    render_reference_impl(doc, true)
}

pub fn render_asa_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    render_reference_impl(doc, false)
}

fn render_reference_impl(doc: &Document, spaces: bool) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_full_authors_first_last(&authors);
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };
    parts.push(format!("{}.", year));

    if !doc.title.is_empty() {
        parts.push(format!("\"{}.\"", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(journal.clone());
    } else if let Some(conference) = &doc.conference {
        parts.push(conference.clone());
    }

    let volume = doc.volume.as_deref().unwrap_or("");
    let issue = doc.issue.as_deref();
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    let vol_str = if spaces {
        let mut s = String::new();
        if !volume.is_empty() {
            s.push_str(volume);
            if let Some(is) = issue {
                s.push_str(&format!(" ({})", is));
            }
            if !pages.is_empty() {
                s.push_str(&format!(": {}", pages));
            }
        }
        s
    } else {
        let mut s = String::new();
        if !volume.is_empty() {
            s.push_str(volume);
            if let Some(is) = issue {
                s.push_str(&format!("({})", is));
            }
            if !pages.is_empty() {
                s.push_str(&format!(":{}", pages));
            }
        }
        s
    };

    if !vol_str.is_empty() {
        parts.push(format!("{}.", vol_str));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("doi: {}.", doi));
    }

    parts.join(" ")
}

pub fn render_apsa_in_text(doc: &Document, _lang: CitationLanguage) -> String {
    render_in_text_impl(doc, ", ")
}

pub fn render_asa_in_text(doc: &Document, _lang: CitationLanguage) -> String {
    render_in_text_impl(doc, ":")
}

fn render_in_text_impl(doc: &Document, page_sep: &str) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let last = last_names(&authors);
    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    let author_part = if last.is_empty() {
        year
    } else if last.len() >= 4 {
        format!("{} et al. {}", last[0], year)
    } else {
        let joined = last.join(", ");
        if last.len() == 1 {
            format!("{} {}", joined, year)
        } else {
            let (l, rest) = last.split_last().unwrap();
            format!("{}, and {} {}", rest.join(", "), l, year)
        }
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    if pages.is_empty() {
        format!("({})", author_part)
    } else {
        format!("({}{}{})", author_part, page_sep, pages)
    }
}

pub fn render_reference(doc: &Document, lang: CitationLanguage, mode: DisplayMode) -> String {
    render_apsa_reference(doc, lang, mode)
}

pub fn render_in_text(doc: &Document, lang: CitationLanguage) -> String {
    render_apsa_in_text(doc, lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Political Behavior".to_string(),
            authors: Some("Smith, John A.; Jones, Bob C.; Brown, David E.".to_string()),
            journal: Some("American Journal".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            volume: Some("42".to_string()),
            issue: Some("3".to_string()),
            page_start: Some("123".to_string()),
            page_end: Some("145".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_apsa_reference() {
        let doc = make_doc();
        let result = render_apsa_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Smith, John A., Bob C. Jones, and David E. Brown."), "apsa authors: {result}");
        assert!(result.contains("2023."), "apsa year: {result}");
        assert!(result.contains("\"Political Behavior.\""), "apsa title in quotes: {result}");
        assert!(result.contains("42 (3): 123-145."), "apsa vol with spaces: {result}");
    }

    #[test]
    fn test_asa_reference() {
        let doc = make_doc();
        let result = render_asa_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("42(3):123-145."), "asa vol no spaces: {result}");
    }

    #[test]
    fn test_apsa_in_text() {
        let doc = make_doc();
        let result = render_apsa_in_text(&doc, CitationLanguage::English);
        assert!(result.contains("(Smith, Jones, and Brown 2023"), "apsa in-text: {result}");
        assert!(result.contains(", 123-145)"), "apsa in-text comma page: {result}");
    }

    #[test]
    fn test_asa_in_text_colon() {
        let doc = make_doc();
        let result = render_asa_in_text(&doc, CitationLanguage::English);
        assert!(result.contains(":123-145)"), "asa in-text colon page: {result}");
    }
}
