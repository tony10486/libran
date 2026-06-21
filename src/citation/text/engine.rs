use anyhow::Result;

use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
use crate::db::documents::{split_authors, Document};

/// Render a full reference-list citation for `doc` in the given style.
///
/// Phase 1 implements only APA 7th. All other styles bail with a
/// "not yet implemented" error.
pub fn render_citation(
    doc: &Document,
    style: CitationStyle,
    _language: CitationLanguage,
    _display_mode: DisplayMode,
) -> Result<String> {
    match style {
        CitationStyle::Apa7th => Ok(render_apa7th_reference(doc)),
        _ => anyhow::bail!("{} style not yet implemented", style.display_name()),
    }
}

/// Render a short in-text citation for `doc` in the given style.
pub fn render_in_text_citation(
    doc: &Document,
    style: CitationStyle,
    _language: CitationLanguage,
) -> Result<String> {
    match style {
        CitationStyle::Apa7th => Ok(render_apa7th_in_text(doc)),
        _ => anyhow::bail!("{} style not yet implemented", style.display_name()),
    }
}

fn render_apa7th_reference(doc: &Document) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(authors_str) = &doc.authors {
        let formatted = format_authors_apa_reference(authors_str);
        if !formatted.is_empty() {
            parts.push(formatted);
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

fn render_apa7th_in_text(doc: &Document) -> String {
    let last_names: Vec<String> = doc
        .authors
        .as_deref()
        .map(split_authors)
        .unwrap_or_default()
        .into_iter()
        .map(|name| parse_author(&name).0)
        .collect();

    let year_part = match doc.pub_year {
        Some(year) => year.to_string(),
        None => "n.d.".to_string(),
    };

    match last_names.len() {
        0 => format!("({})", year_part),
        1 => format!("({}, {})", last_names[0], year_part),
        2 => format!("({} & {}, {})", last_names[0], last_names[1], year_part),
        _ => format!("({} et al., {})", last_names[0], year_part),
    }
}

/// Parses "Last, First", "First Last", or "Last, F." into (last, initial).
fn parse_author(name: &str) -> (String, String) {
    let name = name.trim();
    if name.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim().to_string();
        let first_part = name[comma_pos + 1..].trim();
        let initial = first_initial(first_part);
        (last, initial)
    } else {
        let words: Vec<&str> = name.split_whitespace().collect();
        match words.len() {
            0 => (String::new(), String::new()),
            1 => (words[0].to_string(), String::new()),
            _ => {
                let last = words.last().unwrap().to_string();
                let first_part = words[..words.len() - 1].join(" ");
                let initial = first_initial(&first_part);
                (last, initial)
            }
        }
    }
}

fn first_initial(first: &str) -> String {
    for ch in first.trim().chars() {
        if ch.is_alphabetic() {
            return ch.to_uppercase().collect();
        }
    }
    String::new()
}

/// APA 7th joining: ", & " before the last author (e.g. "Smith, J., & Lee, J.").
fn format_authors_apa_reference(authors: &str) -> String {
    let names: Vec<String> = split_authors(authors)
        .into_iter()
        .map(|name| {
            let (last, initial) = parse_author(&name);
            if initial.is_empty() {
                last
            } else {
                format!("{}, {}.", last, initial)
            }
        })
        .collect();

    match names.len() {
        0 => String::new(),
        1 => names.into_iter().next().unwrap(),
        2 => format!("{}, & {}", names[0], names[1]),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            format!("{}, & {}", rest.join(", "), last)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let in_text = render_in_text_citation(&doc, CitationStyle::Apa7th, CitationLanguage::English)
            .unwrap();
        assert_eq!(in_text, "(Smith & Lee, 2023)");
    }

    #[test]
    fn test_apa_authors_with_and_separator() {
        // Given authors separated by " and " instead of ";"
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
        // Then both authors are parsed and formatted correctly
        assert_eq!(
            result,
            "Smith, J., & Lee, J. (2023). Deep learning. Nature."
        );

        let in_text = render_in_text_citation(&doc, CitationStyle::Apa7th, CitationLanguage::English)
            .unwrap();
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

        let in_text = render_in_text_citation(&doc, CitationStyle::Apa7th, CitationLanguage::English)
            .unwrap();
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
    fn test_unimplemented_style_bails() {
        let doc = doc_with("Deep learning", "Smith, John");
        let result = render_citation(
            &doc,
            CitationStyle::Nature,
            CitationLanguage::English,
            DisplayMode::InText,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not yet implemented"), "got: {err}");
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
        let in_text =
            render_in_text_citation(&doc, CitationStyle::Apa7th, CitationLanguage::English).unwrap();
        assert_eq!(in_text, "(Smith, n.d.)");
    }

    #[test]
    fn test_style_catalog_invariants() {
        assert_eq!(CitationStyle::all().len(), 15);
        assert!(CitationStyle::Apa7th.is_implemented());
        for &style in CitationStyle::all() {
            assert_eq!(style.is_implemented(), style == CitationStyle::Apa7th);
        }
        assert!(CitationStyle::Chicago18NotesBib.is_notes_based());
        assert!(CitationStyle::Chicago18ShortenedNotesBib.is_notes_based());
        assert!(CitationStyle::Mhra4thNotes.is_notes_based());
        assert!(!CitationStyle::Apa7th.is_notes_based());
        assert_eq!(CitationLanguage::all().len(), 4);
    }
}
