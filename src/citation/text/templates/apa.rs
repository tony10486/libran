//! APA 7th edition citation style template.

use crate::citation::text::helpers::{
    format_authors_initials, format_year, get_authors, parse_author,
};
use crate::citation::text::styles::{CitationLanguage, DisplayMode};
use crate::db::documents::Document;

/// Render a full APA 7th reference-list citation.
pub fn render_reference(doc: &Document, _lang: CitationLanguage, _mode: DisplayMode) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let authors = get_authors(Some(authors_str));
        if !authors.is_empty() {
            let formatted = format_authors_apa_reference(&authors);
            if !formatted.is_empty() {
                parts.push(formatted);
            }
        }
    }

    parts.push(match doc.pub_year {
        Some(year) => format!("({}).", year),
        None => "(n.d.).".to_string(),
    });

    if !doc.title.is_empty() {
        parts.push(format!("{}.", doc.title));
    }

    if let Some(journal) = &doc.journal {
        parts.push(format!("{}.", journal));
    } else if let Some(conference) = &doc.conference {
        parts.push(format!("{}.", conference));
    }

    if let Some(doi) = &doc.doi {
        parts.push(format!("https://doi.org/{}", doi));
    }

    parts.join(" ")
}

/// Render a short APA 7th in-text citation.
pub fn render_in_text(doc: &Document, _lang: CitationLanguage) -> String {
    let authors = get_authors(doc.authors.as_deref());
    let last_names: Vec<String> = authors
        .iter()
        .map(|name| parse_author(name, None).0)
        .collect();

    let year_part = format_year(doc.pub_year);

    match last_names.len() {
        0 => format!("({})", year_part),
        1 => format!("({}, {})", last_names[0], year_part),
        2 => format!("({} & {}, {})", last_names[0], last_names[1], year_part),
        _ => format!("({} et al., {})", last_names[0], year_part),
    }
}

/// APA 7th joining: ", & " before the last author (e.g. "Smith, J., & Lee, J.").
fn format_authors_apa_reference(authors: &[String]) -> String {
    format_authors_initials(authors, ", ", ", & ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation::text::engine::render_citation;
    use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
    use crate::db::documents::Document;

    fn doc_with(title: &str, authors: &str) -> Document {
        Document {
            id: Some(1),
            title: title.to_string(),
            authors: Some(authors.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_apa_journal_one_author() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(
            result,
            "Smith, J. (2023). Deep learning. Nature. https://doi.org/10.1234/test"
        );
    }

    #[test]
    fn test_apa_journal_two_authors() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(
            result,
            "Smith, J., & Lee, J. (2023). Deep learning. Nature. https://doi.org/10.1234/test"
        );

        let in_text = render_citation_in_text(&doc);
        assert_eq!(in_text, "(Smith & Lee, 2023)");
    }

    #[test]
    fn test_apa_authors_with_and_separator() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John and Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(
            result,
            "Smith, J., & Lee, J. (2023). Deep learning. Nature."
        );

        let in_text = render_citation_in_text(&doc);
        assert_eq!(in_text, "(Smith & Lee, 2023)");
    }

    #[test]
    fn test_apa_journal_three_authors() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John; Lee, Jane; Brown, Bob".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(
            result,
            "Smith, J., Lee, J., & Brown, B. (2023). Deep learning. Nature."
        );

        let in_text = render_citation_in_text(&doc);
        assert_eq!(in_text, "(Smith et al., 2023)");
    }

    #[test]
    fn test_apa_book_no_journal() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(result, "Smith, J. (2023). Deep learning.");
    }

    #[test]
    fn test_apa_conference_paper() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            conference: Some("ICML".to_string()),
            pub_year: Some(2023),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(result, "Smith, J. (2023). Deep learning. ICML.");
    }

    #[test]
    fn test_apa_missing_fields() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            ..Default::default()
        };
        let result = render_citation(
            &doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
            DisplayMode::InText,
        )
        .unwrap();
        assert_eq!(result, "Smith, J. (n.d.). Deep learning. Nature.");
    }

    #[test]
    fn test_nature_style_now_works() {
        let doc = doc_with("Deep learning", "Smith, John");
        let result = render_citation(
            &doc,
            CitationStyle::Nature,
            CitationLanguage::English,
            DisplayMode::InText,
        );
        assert!(result.is_ok(), "Nature should be implemented: {:?}", result);
    }

    #[test]
    fn test_apa_in_text_no_year() {
        let doc = Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            ..Default::default()
        };
        let in_text = render_citation_in_text(&doc);
        assert_eq!(in_text, "(Smith, n.d.)");
    }

    #[test]
    fn test_style_catalog_invariants() {
        assert_eq!(CitationStyle::all().len(), 15);
        for &style in CitationStyle::all() {
            assert!(style.is_implemented(), "{:?} should be implemented", style);
        }
        assert!(CitationStyle::Chicago18NotesBib.is_notes_based());
        assert!(CitationStyle::Chicago18ShortenedNotesBib.is_notes_based());
        assert!(CitationStyle::Mhra4thNotes.is_notes_based());
        assert!(!CitationStyle::Apa7th.is_notes_based());
        assert_eq!(CitationLanguage::all().len(), 4);
    }

    /// Helper to call render_in_text_citation through the engine.
    fn render_citation_in_text(doc: &Document) -> String {
        crate::citation::text::engine::render_in_text_citation(
            doc,
            CitationStyle::Apa7th,
            CitationLanguage::English,
        )
        .unwrap()
    }
}
