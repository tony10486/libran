//! Golden-file tests for all 15 citation styles across 3 document types.
//!
//! 45 reference tests (15 styles x 3 doc types) + 15 in-text citation tests = 60 tests.

use libran::citation::text::render_citation;
use libran::citation::text::render_in_text_citation;
use libran::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
use libran::db::documents::Document;

// ── Test document factories ──────────────────────────────────────────

fn journal_doc() -> Document {
    Document {
        id: Some(1),
        title: "Deep Learning for NLP".to_string(),
        authors: Some("Smith, John; Lee, Jane".to_string()),
        journal: Some("Nature".to_string()),
        pub_year: Some(2023),
        volume: Some("42".to_string()),
        issue: Some("3".to_string()),
        page_start: Some("123".to_string()),
        page_end: Some("145".to_string()),
        doi: Some("10.1234/test".to_string()),
        abstract_text: Some("A comprehensive survey.".to_string()),
        keywords: Some("machine learning, neural networks".to_string()),
        ..Default::default()
    }
}

fn book_doc() -> Document {
    Document {
        id: Some(1),
        title: "Machine Learning Basics".to_string(),
        authors: Some("Smith, John".to_string()),
        pub_year: Some(2023),
        publisher: Some("MIT Press".to_string()),
        city: Some("Cambridge".to_string()),
        isbn: Some("978-0-123456-78-9".to_string()),
        ..Default::default()
    }
}

fn conference_doc() -> Document {
    Document {
        id: Some(1),
        title: "Advanced Neural Networks".to_string(),
        authors: Some("Brown, Alice; Wilson, Bob".to_string()),
        conference: Some("NeurIPS 2023".to_string()),
        pub_year: Some(2023),
        doi: Some("10.4567/conf".to_string()),
        ..Default::default()
    }
}

fn render_ref(doc: &Document, style: CitationStyle, mode: DisplayMode) -> String {
    render_citation(doc, style, CitationLanguage::English, mode)
        .expect("render_citation should succeed")
}

fn render_intext(doc: &Document, style: CitationStyle) -> String {
    render_in_text_citation(doc, style, CitationLanguage::English)
        .expect("render_in_text_citation should succeed")
}

// ── ACS Guide 2022 ───────────────────────────────────────────────────

#[test]
fn test_acs_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::AcsGuide2022, DisplayMode::InText);
    assert!(out.contains("Smith, J.; Lee, J.;"), "acs authors: {out}");
    assert!(out.contains("*Nature*"), "acs italic journal: {out}");
    assert!(out.contains("**2023**"), "acs bold year: {out}");
    assert!(out.contains("*42*, 123-145"), "acs italic vol+pages: {out}");
    assert!(
        out.contains("DOI: 10.1234/test"),
        "acs doi uppercase: {out}"
    );
}

#[test]
fn test_acs_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::AcsGuide2022, DisplayMode::InText);
    assert!(out.contains("Smith, J.;"), "acs book author: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "acs book title: {out}"
    );
    assert!(out.contains("**2023**"), "acs book bold year: {out}");
    assert!(!out.contains("DOI:"), "acs book no doi: {out}");
}

#[test]
fn test_acs_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::AcsGuide2022, DisplayMode::InText);
    assert!(
        out.contains("Brown, A.; Wilson, B.;"),
        "acs conf authors: {out}"
    );
    assert!(out.contains("*NeurIPS 2023*"), "acs conf italic: {out}");
    assert!(out.contains("DOI: 10.4567/conf"), "acs conf doi: {out}");
}

#[test]
fn test_acs_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::AcsGuide2022);
    assert_eq!(out, "[1]", "acs in-text numeric: {out}");
}

// ── AMA 11th ─────────────────────────────────────────────────────────

#[test]
fn test_ama_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Ama11th, DisplayMode::InText);
    assert!(
        out.contains("Smith J, Lee J."),
        "ama authors no periods: {out}"
    );
    assert!(
        out.contains("2023;42(3):123-145."),
        "ama year;vol(issue):pages: {out}"
    );
    assert!(
        out.contains("doi: 10.1234/test"),
        "ama lowercase doi: {out}"
    );
}

#[test]
fn test_ama_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Ama11th, DisplayMode::InText);
    assert!(out.contains("Smith J."), "ama book author: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "ama book title: {out}"
    );
    assert!(out.contains("2023."), "ama book year: {out}");
}

#[test]
fn test_ama_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Ama11th, DisplayMode::InText);
    assert!(
        out.contains("Brown A, Wilson B."),
        "ama conf authors: {out}"
    );
    assert!(out.contains("NeurIPS 2023."), "ama conf name: {out}");
    assert!(out.contains("doi: 10.4567/conf"), "ama conf doi: {out}");
}

#[test]
fn test_ama_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Ama11th);
    assert_eq!(out, "[1]", "ama in-text numeric: {out}");
}

// ── APA 7th ──────────────────────────────────────────────────────────

#[test]
fn test_apa_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Apa7th, DisplayMode::InText);
    assert!(
        out.contains("Smith, J., & Lee, J."),
        "apa authors with ampersand: {out}"
    );
    assert!(out.contains("(2023)."), "apa year in parens: {out}");
    assert!(
        out.contains("https://doi.org/10.1234/test"),
        "apa doi url: {out}"
    );
}

#[test]
fn test_apa_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Apa7th, DisplayMode::InText);
    assert!(out.contains("Smith, J."), "apa book author: {out}");
    assert!(out.contains("(2023)."), "apa book year: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "apa book title: {out}"
    );
}

#[test]
fn test_apa_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Apa7th, DisplayMode::InText);
    assert!(
        out.contains("Brown, A., & Wilson, B."),
        "apa conf authors: {out}"
    );
    assert!(out.contains("NeurIPS 2023."), "apa conf name: {out}");
    assert!(
        out.contains("https://doi.org/10.4567/conf"),
        "apa conf doi url: {out}"
    );
}

#[test]
fn test_apa_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Apa7th);
    assert_eq!(out, "(Smith & Lee, 2023)", "apa in-text: {out}");
}

// ── APSA 2018 ────────────────────────────────────────────────────────

#[test]
fn test_apsa_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Apsa2018, DisplayMode::InText);
    assert!(
        out.contains("Smith, John, and Jane Lee."),
        "apsa full names: {out}"
    );
    assert!(
        out.contains("\"Deep Learning for NLP.\""),
        "apsa title in quotes: {out}"
    );
    assert!(
        out.contains("42 (3): 123-145."),
        "apsa vol with spaces: {out}"
    );
}

#[test]
fn test_apsa_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Apsa2018, DisplayMode::InText);
    assert!(out.contains("Smith, John."), "apsa book author: {out}");
    assert!(
        out.contains("\"Machine Learning Basics.\""),
        "apsa book title: {out}"
    );
    assert!(out.contains("2023."), "apsa book year: {out}");
}

#[test]
fn test_apsa_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Apsa2018, DisplayMode::InText);
    assert!(
        out.contains("Brown, Alice, and Bob Wilson."),
        "apsa conf authors: {out}"
    );
    assert!(
        out.contains("\"Advanced Neural Networks.\""),
        "apsa conf title: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "apsa conf name: {out}");
}

#[test]
fn test_apsa_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Apsa2018);
    assert!(
        out.contains("Smith, and Lee 2023"),
        "apsa in-text author-date: {out}"
    );
    assert!(out.contains(", 123-145"), "apsa in-text comma page: {out}");
}

// ── ASA 6th/7th ──────────────────────────────────────────────────────

#[test]
fn test_asa_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Asa6th7th, DisplayMode::InText);
    assert!(
        out.contains("Smith, John, and Jane Lee."),
        "asa full names: {out}"
    );
    assert!(out.contains("42(3):123-145."), "asa vol no spaces: {out}");
    assert!(
        out.contains("\"Deep Learning for NLP.\""),
        "asa title in quotes: {out}"
    );
}

#[test]
fn test_asa_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Asa6th7th, DisplayMode::InText);
    assert!(out.contains("Smith, John."), "asa book author: {out}");
    assert!(
        out.contains("\"Machine Learning Basics.\""),
        "asa book title: {out}"
    );
    assert!(out.contains("2023."), "asa book year: {out}");
}

#[test]
fn test_asa_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Asa6th7th, DisplayMode::InText);
    assert!(
        out.contains("Brown, Alice, and Bob Wilson."),
        "asa conf authors: {out}"
    );
    assert!(
        out.contains("\"Advanced Neural Networks.\""),
        "asa conf title: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "asa conf name: {out}");
}

#[test]
fn test_asa_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Asa6th7th);
    assert!(
        out.contains("Smith, and Lee 2023"),
        "asa in-text author-date: {out}"
    );
    assert!(out.contains(":123-145"), "asa in-text colon page: {out}");
}

// ── Chicago 18th Author-Date ─────────────────────────────────────────

#[test]
fn test_chicago_ad_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18AuthorDate,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Smith, John, and Jane Lee."),
        "chicago ad authors: {out}"
    );
    assert!(
        out.contains("42, no. 3 (2023): 123-145."),
        "chicago ad vol/issue: {out}"
    );
    assert!(
        out.contains("https://doi.org/10.1234/test"),
        "chicago ad doi: {out}"
    );
}

#[test]
fn test_chicago_ad_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18AuthorDate,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Smith, John."),
        "chicago ad book author: {out}"
    );
    assert!(
        out.contains("\"Machine Learning Basics.\""),
        "chicago ad book title: {out}"
    );
    assert!(out.contains("2023."), "chicago ad book year: {out}");
}

#[test]
fn test_chicago_ad_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18AuthorDate,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Brown, Alice, and Bob Wilson."),
        "chicago ad conf authors: {out}"
    );
    assert!(
        out.contains("\"Advanced Neural Networks.\""),
        "chicago ad conf title: {out}"
    );
    assert!(
        out.contains("https://doi.org/10.4567/conf"),
        "chicago ad conf doi: {out}"
    );
}

#[test]
fn test_chicago_ad_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Chicago18AuthorDate);
    assert!(
        out.contains("Smith, and Lee 2023"),
        "chicago ad in-text: {out}"
    );
    assert!(out.contains(", 123-145"), "chicago ad in-text page: {out}");
}

// ── Chicago 18th Notes+Bib ───────────────────────────────────────────

#[test]
fn test_chicago_nb_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18NotesBib,
        DisplayMode::Footnotes,
    );
    assert!(out.starts_with("1. "), "chicago nb footnote prefix: {out}");
    assert!(
        out.contains("John Smith, and Jane Lee"),
        "chicago nb first-name-first: {out}"
    );
    assert!(
        out.contains("\"Deep Learning for NLP\""),
        "chicago nb title: {out}"
    );
    assert!(
        out.contains("42, no. 3 (2023): 123-145"),
        "chicago nb vol/issue: {out}"
    );
}

#[test]
fn test_chicago_nb_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18NotesBib,
        DisplayMode::Footnotes,
    );
    assert!(out.starts_with("1. "), "chicago nb book footnote: {out}");
    assert!(
        out.contains("John Smith"),
        "chicago nb book first-name-first: {out}"
    );
    assert!(
        out.contains("\"Machine Learning Basics\""),
        "chicago nb book title: {out}"
    );
}

#[test]
fn test_chicago_nb_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18NotesBib,
        DisplayMode::Footnotes,
    );
    assert!(out.starts_with("1. "), "chicago nb conf footnote: {out}");
    assert!(
        out.contains("Alice Brown, and Bob Wilson"),
        "chicago nb conf first-name-first: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "chicago nb conf name: {out}");
}

#[test]
fn test_chicago_nb_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Chicago18NotesBib);
    assert_eq!(out, "[1]", "chicago nb in-text marker: {out}");
}

// ── Chicago 18th Shortened Notes+Bib ─────────────────────────────────

#[test]
fn test_chicago_shortened_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18ShortenedNotesBib,
        DisplayMode::Footnotes,
    );
    assert!(
        out.starts_with("1. "),
        "chicago shortened footnote prefix: {out}"
    );
    assert!(
        out.contains("John Smith, and Jane Lee"),
        "chicago shortened first-name-first: {out}"
    );
    assert!(
        out.contains("\"Deep Learning for NLP\""),
        "chicago shortened title: {out}"
    );
    assert!(
        out.contains("42, no. 3 (2023): 123-145"),
        "chicago shortened vol/issue: {out}"
    );
}

#[test]
fn test_chicago_shortened_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18ShortenedNotesBib,
        DisplayMode::Footnotes,
    );
    assert!(
        out.starts_with("1. "),
        "chicago shortened book footnote: {out}"
    );
    assert!(
        out.contains("John Smith"),
        "chicago shortened book first-name-first: {out}"
    );
    assert!(
        out.contains("\"Machine Learning Basics\""),
        "chicago shortened book title: {out}"
    );
}

#[test]
fn test_chicago_shortened_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::Chicago18ShortenedNotesBib,
        DisplayMode::Footnotes,
    );
    assert!(
        out.starts_with("1. "),
        "chicago shortened conf footnote: {out}"
    );
    assert!(
        out.contains("Alice Brown, and Bob Wilson"),
        "chicago shortened conf authors: {out}"
    );
    assert!(
        out.contains("NeurIPS 2023"),
        "chicago shortened conf name: {out}"
    );
}

#[test]
fn test_chicago_shortened_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Chicago18ShortenedNotesBib);
    assert_eq!(out, "[1]", "chicago shortened in-text marker: {out}");
}

// ── Cite Them Right 12th Harvard ─────────────────────────────────────

#[test]
fn test_ctr_harvard_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::CiteThemRight12thHarvard,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Smith, J. and Lee, J."),
        "ctr harvard initials no space: {out}"
    );
    assert!(
        out.contains("(2023) 'Deep Learning for NLP'"),
        "ctr harvard year+single-quote title: {out}"
    );
    assert!(
        out.contains("42(3), pp. 123-145"),
        "ctr harvard vol/pages: {out}"
    );
}

#[test]
fn test_ctr_harvard_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::CiteThemRight12thHarvard,
        DisplayMode::InText,
    );
    assert!(out.contains("Smith, J."), "ctr harvard book author: {out}");
    assert!(
        out.contains("'Machine Learning Basics'"),
        "ctr harvard book single-quote title: {out}"
    );
    assert!(out.contains("(2023)"), "ctr harvard book year: {out}");
}

#[test]
fn test_ctr_harvard_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::CiteThemRight12thHarvard,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Brown, A. and Wilson, B."),
        "ctr harvard conf authors: {out}"
    );
    assert!(
        out.contains("'Advanced Neural Networks'"),
        "ctr harvard conf single-quote title: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "ctr harvard conf name: {out}");
}

#[test]
fn test_ctr_harvard_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::CiteThemRight12thHarvard);
    assert!(
        out.contains("Smith and Lee 2023"),
        "ctr harvard in-text: {out}"
    );
    assert!(
        out.contains("p. 123-145"),
        "ctr harvard in-text page: {out}"
    );
}

// ── Elsevier Harvard with Titles ─────────────────────────────────────

#[test]
fn test_elsevier_harvard_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::ElsevierHarvardWithTitles,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Smith, J., Lee, J.,"),
        "elsevier no 'and': {out}"
    );
    assert!(out.contains("2023."), "elsevier year with period: {out}");
    assert!(out.contains("42, 123-145."), "elsevier vol/pages: {out}");
}

#[test]
fn test_elsevier_harvard_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::ElsevierHarvardWithTitles,
        DisplayMode::InText,
    );
    assert!(out.contains("Smith, J.,"), "elsevier book author: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "elsevier book title: {out}"
    );
    assert!(out.contains("2023."), "elsevier book year: {out}");
}

#[test]
fn test_elsevier_harvard_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::ElsevierHarvardWithTitles,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Brown, A., Wilson, B.,"),
        "elsevier conf authors no 'and': {out}"
    );
    assert!(
        out.contains("Advanced Neural Networks."),
        "elsevier conf title: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "elsevier conf name: {out}");
}

#[test]
fn test_elsevier_harvard_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::ElsevierHarvardWithTitles);
    assert!(
        out.contains("Smith and Lee 2023"),
        "elsevier in-text: {out}"
    );
    assert!(out.contains("p. 123-145"), "elsevier in-text page: {out}");
}

// ── IEEE v11.29.2023 ─────────────────────────────────────────────────

#[test]
fn test_ieee_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::IeeeV11_29_2023, DisplayMode::InText);
    assert!(
        out.contains("J. Smith and J. Lee,"),
        "ieee initials first: {out}"
    );
    assert!(
        out.contains("\"Deep Learning for NLP\","),
        "ieee title in quotes: {out}"
    );
    assert!(out.contains("vol. 42"), "ieee vol: {out}");
    assert!(out.contains("no. 3"), "ieee no: {out}");
    assert!(out.contains("pp. 123-145"), "ieee pp: {out}");
}

#[test]
fn test_ieee_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::IeeeV11_29_2023, DisplayMode::InText);
    assert!(out.contains("J. Smith,"), "ieee book initials first: {out}");
    assert!(
        out.contains("\"Machine Learning Basics\","),
        "ieee book title in quotes: {out}"
    );
    assert!(out.contains("2023."), "ieee book year: {out}");
}

#[test]
fn test_ieee_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::IeeeV11_29_2023, DisplayMode::InText);
    assert!(
        out.contains("A. Brown and B. Wilson,"),
        "ieee conf initials first: {out}"
    );
    assert!(
        out.contains("\"Advanced Neural Networks\","),
        "ieee conf title: {out}"
    );
    assert!(out.contains("NeurIPS 2023,"), "ieee conf name: {out}");
}

#[test]
fn test_ieee_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::IeeeV11_29_2023);
    assert_eq!(out, "[1]", "ieee in-text numeric: {out}");
}

// ── MHRA 4th Notes ───────────────────────────────────────────────────

#[test]
fn test_mhra_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Mhra4thNotes, DisplayMode::Footnotes);
    assert!(out.starts_with("1. "), "mhra footnote prefix: {out}");
    assert!(
        out.contains("John Smith and Jane Lee"),
        "mhra first-name-first: {out}"
    );
    assert!(
        out.contains("'Deep Learning for NLP'"),
        "mhra single-quote title: {out}"
    );
    assert!(out.contains("42.3"), "mhra volume.issue format: {out}");
}

#[test]
fn test_mhra_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Mhra4thNotes, DisplayMode::Footnotes);
    assert!(out.starts_with("1. "), "mhra book footnote: {out}");
    assert!(
        out.contains("John Smith"),
        "mhra book first-name-first: {out}"
    );
    assert!(
        out.contains("'Machine Learning Basics'"),
        "mhra book single-quote title: {out}"
    );
}

#[test]
fn test_mhra_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Mhra4thNotes, DisplayMode::Footnotes);
    assert!(out.starts_with("1. "), "mhra conf footnote: {out}");
    assert!(
        out.contains("Alice Brown and Bob Wilson"),
        "mhra conf first-name-first: {out}"
    );
    assert!(
        out.contains("'Advanced Neural Networks'"),
        "mhra conf single-quote title: {out}"
    );
}

#[test]
fn test_mhra_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Mhra4thNotes);
    assert_eq!(out, "[1]", "mhra in-text marker: {out}");
}

// ── MLA 9th In-Text ──────────────────────────────────────────────────

#[test]
fn test_mla_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Mla9thInText, DisplayMode::InText);
    assert!(
        out.contains("Smith, John and Jane Lee."),
        "mla two authors full names: {out}"
    );
    assert!(
        out.contains("\"Deep Learning for NLP.\""),
        "mla title in quotes: {out}"
    );
    assert!(out.contains("vol. 42"), "mla vol: {out}");
    assert!(out.contains("no. 3"), "mla no: {out}");
    assert!(out.contains("pp. 123-145"), "mla pp: {out}");
}

#[test]
fn test_mla_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Mla9thInText, DisplayMode::InText);
    assert!(out.contains("Smith, John."), "mla book author: {out}");
    assert!(
        out.contains("\"Machine Learning Basics.\""),
        "mla book title: {out}"
    );
    assert!(out.contains("2023."), "mla book year: {out}");
}

#[test]
fn test_mla_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Mla9thInText, DisplayMode::InText);
    assert!(
        out.contains("Brown, Alice and Bob Wilson."),
        "mla conf authors: {out}"
    );
    assert!(
        out.contains("\"Advanced Neural Networks.\""),
        "mla conf title: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "mla conf name: {out}");
}

#[test]
fn test_mla_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Mla9thInText);
    assert!(out.contains("Smith and Lee"), "mla in-text author: {out}");
    assert!(out.contains("123-145"), "mla in-text page: {out}");
    assert!(!out.contains("2023"), "mla in-text NO year: {out}");
}

// ── Nature ───────────────────────────────────────────────────────────

#[test]
fn test_nature_journal() {
    let doc = journal_doc();
    let out = render_ref(&doc, CitationStyle::Nature, DisplayMode::InText);
    assert!(
        out.contains("Smith, J. & Lee, J."),
        "nature authors with ampersand: {out}"
    );
    assert!(
        out.contains("42, 123-145 (2023)"),
        "nature vol pages year: {out}"
    );
    assert!(out.contains("doi: 10.1234/test"), "nature doi: {out}");
}

#[test]
fn test_nature_book() {
    let doc = book_doc();
    let out = render_ref(&doc, CitationStyle::Nature, DisplayMode::InText);
    assert!(out.contains("Smith, J."), "nature book author: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "nature book title: {out}"
    );
    assert!(out.contains("(2023)"), "nature book year in parens: {out}");
}

#[test]
fn test_nature_conference() {
    let doc = conference_doc();
    let out = render_ref(&doc, CitationStyle::Nature, DisplayMode::InText);
    assert!(
        out.contains("Brown, A. & Wilson, B."),
        "nature conf authors: {out}"
    );
    assert!(out.contains("NeurIPS 2023"), "nature conf name: {out}");
    assert!(out.contains("(2023)"), "nature conf year: {out}");
}

#[test]
fn test_nature_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::Nature);
    assert_eq!(out, "[1]", "nature in-text numeric: {out}");
}

// ── NLM/Vancouver Citing Medicine 2nd ────────────────────────────────

#[test]
fn test_vancouver_journal() {
    let doc = journal_doc();
    let out = render_ref(
        &doc,
        CitationStyle::NlmVancouverCitingMedicine2nd,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Smith J, Lee J."),
        "vancouver authors no periods: {out}"
    );
    assert!(
        out.contains("2023;42(3):123-145."),
        "vancouver year;vol(issue):pages: {out}"
    );
    assert!(out.contains("doi: 10.1234/test"), "vancouver doi: {out}");
}

#[test]
fn test_vancouver_book() {
    let doc = book_doc();
    let out = render_ref(
        &doc,
        CitationStyle::NlmVancouverCitingMedicine2nd,
        DisplayMode::InText,
    );
    assert!(out.contains("Smith J."), "vancouver book author: {out}");
    assert!(
        out.contains("Machine Learning Basics."),
        "vancouver book title: {out}"
    );
    assert!(out.contains("2023."), "vancouver book year: {out}");
}

#[test]
fn test_vancouver_conference() {
    let doc = conference_doc();
    let out = render_ref(
        &doc,
        CitationStyle::NlmVancouverCitingMedicine2nd,
        DisplayMode::InText,
    );
    assert!(
        out.contains("Brown A, Wilson B."),
        "vancouver conf authors: {out}"
    );
    assert!(out.contains("NeurIPS 2023."), "vancouver conf name: {out}");
    assert!(
        out.contains("doi: 10.4567/conf"),
        "vancouver conf doi: {out}"
    );
}

#[test]
fn test_vancouver_in_text() {
    let doc = journal_doc();
    let out = render_intext(&doc, CitationStyle::NlmVancouverCitingMedicine2nd);
    assert_eq!(out, "(1)", "vancouver in-text parenthesized numeric: {out}");
}
