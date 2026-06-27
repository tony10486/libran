//! Cite Them Right 12th Harvard and Elsevier Harvard with titles citation style templates.
//!
//! CTR Harvard: Smith, J.A., Jones, B.C. and Brown, D.E. (2023) 'Title', Journal, 42(3), pp. 123-145.
//! Elsevier Harvard: Smith, J.A., Jones, B.C., Brown, D.E., 2023. Title. Journal 42, 123-145.

use crate::citation::text::helpers::{format_pages, get_authors, parse_author_full};
use crate::citation::text::locale::{Term, term};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_harvard_initials(authors: &[String]) -> Vec<String> {
    authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name);
            if first.is_empty() {
                return last;
            }
            let initials: String = first
                .split_whitespace()
                .filter_map(|w| w.chars().next().filter(|c| c.is_alphabetic()))
                .map(|c| format!("{}.", c.to_uppercase().collect::<String>()))
                .collect::<Vec<_>>()
                .join("");
            if initials.is_empty() {
                last
            } else {
                format!("{}, {}", last, initials)
            }
        })
        .collect()
}

fn last_names(authors: &[String]) -> Vec<String> {
    authors.iter().map(|n| parse_author_full(n).0).collect()
}

pub fn render_ctr_harvard_reference(
    doc: &Document,
    lang: CitationLanguage,
    _mode: DisplayMode,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted_list = format_harvard_initials(&authors);
        let author_str = if authors.len() >= 4 {
            format!("{} {}", formatted_list[0], term(Term::EtAl, lang))
        } else {
            let (last, rest) = formatted_list.split_last().unwrap();
            if rest.is_empty() {
                last.clone()
            } else {
                format!("{} {} {}", rest.join(", "), term(Term::And, lang), last)
            }
        };
        if !author_str.is_empty() {
            parts.push(author_str);
        }
    }

    let year = match doc.pub_year {
        Some(y) => format!("({})", y),
        None => "(n.d.)".to_string(),
    };

    if !doc.title.is_empty() {
        parts.push(format!("{} '{}',", year, doc.title));
    } else {
        parts.push(format!("{},", year));
    }

    if let Some(journal) = &doc.journal {
        parts.push(format!("{},", journal));
    } else if let Some(conference) = &doc.conference {
        parts.push(format!("{},", conference));
    }

    let volume = doc.volume.as_deref().unwrap_or("");
    let issue = doc.issue.as_deref();
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    if !volume.is_empty() {
        let mut vip = volume.to_string();
        if let Some(is) = issue {
            vip.push_str(&format!("({})", is));
        }
        if !pages.is_empty() {
            vip.push_str(&format!(", pp. {}", pages));
        }
        parts.push(format!("{}.", vip));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("doi: {}", doi));
    }

    parts.join(" ")
}

pub fn render_ctr_harvard_in_text(doc: &Document, lang: CitationLanguage) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let last = last_names(&authors);
    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    let author_part = if last.is_empty() {
        year
    } else if last.len() >= 4 {
        format!("{} {} {}", last[0], term(Term::EtAl, lang), year)
    } else if last.len() == 1 {
        format!("{}", year)
    } else {
        let (l, rest) = last.split_last().unwrap();
        format!(
            "{} {} {} {}",
            rest.join(", "),
            term(Term::And, lang),
            l,
            year
        )
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    if pages.is_empty() {
        format!("({})", author_part)
    } else {
        format!("({}, p. {})", author_part, pages)
    }
}

pub fn render_elsevier_harvard_reference(
    doc: &Document,
    _lang: CitationLanguage,
    _mode: DisplayMode,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted_list = format_harvard_initials(&authors);
        let joined = formatted_list.join(", ");
        if !joined.is_empty() {
            parts.push(format!("{},", joined));
        }
    }

    let year = match doc.pub_year {
        Some(y) => format!("{}.", y),
        None => "n.d.".to_string(),
    };
    parts.push(year);

    if !doc.title.is_empty() {
        parts.push(format!("{}.", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(journal.clone());
    } else if let Some(conference) = &doc.conference {
        parts.push(conference.clone());
    }

    let volume = doc.volume.as_deref().unwrap_or("");
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());

    if !volume.is_empty() && !pages.is_empty() {
        parts.push(format!("{}, {}.", volume, pages));
    } else if !volume.is_empty() {
        parts.push(format!("{}.", volume));
    } else if !pages.is_empty() {
        parts.push(format!("{}.", pages));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("doi: {}", doi));
    }

    parts.join(" ")
}

pub fn render_elsevier_harvard_in_text(doc: &Document, lang: CitationLanguage) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let last = last_names(&authors);
    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    let author_part = if last.is_empty() {
        year
    } else if last.len() >= 3 {
        format!("{} {} {}", last[0], term(Term::EtAl, lang), year)
    } else if last.len() == 1 {
        format!("{} {}", last[0], year)
    } else {
        let (l, rest) = last.split_last().unwrap();
        format!("{} and {} {}", rest.join(", "), l, year)
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    if pages.is_empty() {
        format!("({})", author_part)
    } else {
        format!("({}, p. {})", author_part, pages)
    }
}

pub fn render_reference(doc: &Document, lang: CitationLanguage, mode: DisplayMode) -> String {
    render_ctr_harvard_reference(doc, lang, mode)
}

pub fn render_in_text(doc: &Document, lang: CitationLanguage) -> String {
    render_ctr_harvard_in_text(doc, lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Research Methods".to_string(),
            authors: Some("Smith, John A.; Jones, Bob C.; Brown, David E.".to_string()),
            journal: Some("Social Science Journal".to_string()),
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
    fn test_ctr_harvard_reference() {
        let doc = make_doc();
        let result =
            render_ctr_harvard_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(
            result.contains("and Brown"),
            "ctr harvard 'and' before last: {result}"
        );
        assert!(
            result.contains("(2023)"),
            "ctr harvard year in parens: {result}"
        );
        assert!(
            result.contains("'Research Methods'"),
            "ctr harvard single quotes: {result}"
        );
        assert!(
            result.contains("42(3), pp. 123-145"),
            "ctr harvard vol/pages: {result}"
        );
    }

    #[test]
    fn test_elsevier_harvard_reference() {
        let doc = make_doc();
        let result =
            render_elsevier_harvard_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(!result.contains(" and "), "elsevier no 'and': {result}");
        assert!(
            result.contains("2023."),
            "elsevier year with period: {result}"
        );
        assert!(
            result.contains("42, 123-145."),
            "elsevier vol/pages: {result}"
        );
    }

    #[test]
    fn test_elsevier_harvard_three_plus_et_al_in_text() {
        let doc = make_doc();
        let result = render_elsevier_harvard_in_text(&doc, CitationLanguage::English);
        assert!(
            result.contains("et al."),
            "elsevier 3+ et al in-text: {result}"
        );
    }
}
