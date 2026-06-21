//! Chicago 18th edition citation style templates (3 variants).
//!
//! Author-Date: Smith, John A., Bob C. Jones, and David E. Brown. 2023. "Title." Journal 42, no. 3: 123-45.
//! Notes-Bib (footnote): 1. John A. Smith, Bob C. Jones, and David E. Brown, "Title," Journal 42, no. 3 (2023): 123-45.
//! Notes-Bib (bibliography): Smith, John A., Bob C. Jones, and David E. Brown. "Title." Journal 42, no. 3 (2023): 123-45.
//! Shortened: same as Notes-Bib for single-doc.

use crate::citation::text::helpers::{
    format_pages, get_authors, parse_author_full,
};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

fn format_bib_authors(authors: &[String], _and_word: &str) -> String {
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

fn format_note_authors(authors: &[String], _and_word: &str) -> String {
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

fn vol_issue_year_pages(volume: &str, issue: Option<&str>, year: &str, pages: &str) -> String {
    let mut s = String::new();
    if !volume.is_empty() {
        s.push_str(volume);
        if let Some(is) = issue {
            s.push_str(&format!(", no. {}", is));
        }
        s.push_str(&format!(" ({})", year));
        if !pages.is_empty() {
            s.push_str(&format!(": {}", pages));
        }
    } else {
        s.push_str(&format!("({})", year));
        if !pages.is_empty() {
            s.push_str(&format!(": {}", pages));
        }
    }
    s
}

pub fn render_author_date_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_bib_authors(&authors, "and");
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
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    let viyp = vol_issue_year_pages(volume, doc.issue.as_deref(), &year, &pages);
    if !viyp.is_empty() {
        parts.push(format!("{}.", viyp));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("https://doi.org/{}", doi));
    }

    parts.join(" ")
}

pub fn render_author_date_in_text(doc: &Document, _lang: CitationLanguage) -> String {
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
    } else if last.len() == 1 {
        format!("{} {}", last[0], year)
    } else {
        let (l, rest) = last.split_last().unwrap();
        format!("{}, and {} {}", rest.join(", "), l, year)
    };

    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    if pages.is_empty() {
        format!("({})", author_part)
    } else {
        format!("({}, {})", author_part, pages)
    }
}

fn render_note_impl(doc: &Document, mode: DisplayMode) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let author_str = format_note_authors(&authors, "and");

    let year = match doc.pub_year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    };

    let title = if !doc.title.is_empty() {
        format!("\"{}\"", doc.title)
    } else {
        String::new()
    };

    let journal = doc.journal.clone().or(doc.conference.clone()).unwrap_or_default();
    let volume = doc.volume.as_deref().unwrap_or("");
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    let viyp = vol_issue_year_pages(volume, doc.issue.as_deref(), &year, &pages);

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
    if !viyp.is_empty() {
        elements.push(viyp);
    }

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
        DisplayMode::InText => render_bibliography(doc),
    }
}

fn render_bibliography(doc: &Document) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        let formatted = format_bib_authors(&authors, "and");
        if !formatted.is_empty() {
            parts.push(format!("{}.", formatted));
        }
    }

    if !doc.title.is_empty() {
        parts.push(format!("\"{}.\"", doc.title));
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
    let volume = doc.volume.as_deref().unwrap_or("");
    let pages = format_pages(doc.page_start.as_deref(), doc.page_end.as_deref());
    let viyp = vol_issue_year_pages(volume, doc.issue.as_deref(), &year, &pages);
    if !viyp.is_empty() {
        parts.push(format!("{}.", viyp));
    }

    parts.join(" ")
}

pub fn render_notes_bib_reference(doc: &Document, _lang: CitationLanguage, mode: DisplayMode) -> String {
    render_note_impl(doc, mode)
}

pub fn render_notes_bib_in_text(doc: &Document, _lang: CitationLanguage) -> String {
    let num = doc.id.unwrap_or(1) as usize;
    format!("[{}]", num)
}

pub fn render_shortened_notes_reference(doc: &Document, lang: CitationLanguage, mode: DisplayMode) -> String {
    render_notes_bib_reference(doc, lang, mode)
}

pub fn render_shortened_notes_in_text(doc: &Document, lang: CitationLanguage) -> String {
    render_notes_bib_in_text(doc, lang)
}

pub fn render_reference(doc: &Document, lang: CitationLanguage, mode: DisplayMode) -> String {
    render_author_date_reference(doc, lang, mode)
}

pub fn render_in_text(doc: &Document, lang: CitationLanguage) -> String {
    render_author_date_in_text(doc, lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            title: "Historical Analysis".to_string(),
            authors: Some("Smith, John A.; Jones, Bob C.; Brown, David E.".to_string()),
            journal: Some("Journal of History".to_string()),
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
    fn test_chicago_ad_reference() {
        let doc = make_doc();
        let result = render_author_date_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Smith, John A., Bob C. Jones, and David E. Brown."), "chicago ad authors: {result}");
        assert!(result.contains("2023."), "chicago ad year: {result}");
        assert!(result.contains("\"Historical Analysis.\""), "chicago ad title: {result}");
        assert!(result.contains("42, no. 3 (2023): 123-145."), "chicago ad vol/issue: {result}");
    }

    #[test]
    fn test_chicago_ad_in_text() {
        let doc = make_doc();
        let result = render_author_date_in_text(&doc, CitationLanguage::English);
        assert!(result.contains("(Smith, Jones, and Brown 2023"), "chicago ad in-text: {result}");
        assert!(result.contains(", 123-145)"), "chicago ad in-text page: {result}");
    }

    #[test]
    fn test_chicago_nb_footnote() {
        let doc = make_doc();
        let result = render_notes_bib_reference(&doc, CitationLanguage::English, DisplayMode::Footnotes);
        assert!(result.starts_with("1. "), "chicago nb footnote starts with number: {result}");
        assert!(result.contains("John A. Smith"), "chicago nb footnote first-name-first: {result}");
        assert!(result.contains("\"Historical Analysis\""), "chicago nb title: {result}");
    }

    #[test]
    fn test_chicago_nb_bibliography() {
        let doc = make_doc();
        let result = render_notes_bib_reference(&doc, CitationLanguage::English, DisplayMode::InText);
        assert!(result.contains("Smith, John A."), "chicago nb bib last-name-first: {result}");
    }

    #[test]
    fn test_chicago_nb_endnote() {
        let doc = make_doc();
        let result = render_notes_bib_reference(&doc, CitationLanguage::English, DisplayMode::Endnotes);
        assert!(result.starts_with("[1] "), "chicago nb endnote bracket: {result}");
    }

    #[test]
    fn test_chicago_nb_in_text_marker() {
        let doc = make_doc();
        assert_eq!(render_notes_bib_in_text(&doc, CitationLanguage::English), "[1]");
    }
}
