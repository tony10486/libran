use anyhow::Result;

use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
use crate::citation::text::templates;
use crate::db::documents::Document;

pub fn render_citation(
    doc: &Document,
    style: CitationStyle,
    language: CitationLanguage,
    display_mode: DisplayMode,
) -> Result<String> {
    let result = match style {
        CitationStyle::AcsGuide2022 => {
            templates::acs::render_reference(doc, language, display_mode)
        }
        CitationStyle::Ama11th => templates::ama::render_reference(doc, language, display_mode),
        CitationStyle::Apa7th => templates::apa::render_reference(doc, language, display_mode),
        CitationStyle::Apsa2018 => {
            templates::apsa_asa::render_apsa_reference(doc, language, display_mode)
        }
        CitationStyle::Asa6th7th => {
            templates::apsa_asa::render_asa_reference(doc, language, display_mode)
        }
        CitationStyle::Chicago18AuthorDate => {
            templates::chicago::render_author_date_reference(doc, language, display_mode)
        }
        CitationStyle::Chicago18NotesBib => {
            templates::chicago::render_notes_bib_reference(doc, language, display_mode)
        }
        CitationStyle::Chicago18ShortenedNotesBib => {
            templates::chicago::render_shortened_notes_reference(doc, language, display_mode)
        }
        CitationStyle::CiteThemRight12thHarvard => {
            templates::harvard::render_ctr_harvard_reference(doc, language, display_mode)
        }
        CitationStyle::ElsevierHarvardWithTitles => {
            templates::harvard::render_elsevier_harvard_reference(doc, language, display_mode)
        }
        CitationStyle::IeeeV11_29_2023 => {
            templates::ieee::render_reference(doc, language, display_mode)
        }
        CitationStyle::Mhra4thNotes => {
            templates::mhra::render_reference(doc, language, display_mode)
        }
        CitationStyle::Mla9thInText => {
            templates::mla::render_reference(doc, language, display_mode)
        }
        CitationStyle::Nature => templates::nature::render_reference(doc, language, display_mode),
        CitationStyle::NlmVancouverCitingMedicine2nd => {
            templates::vancouver::render_reference(doc, language, display_mode)
        }
    };
    Ok(result)
}

pub fn render_in_text_citation(
    doc: &Document,
    style: CitationStyle,
    language: CitationLanguage,
) -> Result<String> {
    let result = match style {
        CitationStyle::AcsGuide2022 => templates::acs::render_in_text(doc, language),
        CitationStyle::Ama11th => templates::ama::render_in_text(doc, language),
        CitationStyle::Apa7th => templates::apa::render_in_text(doc, language),
        CitationStyle::Apsa2018 => templates::apsa_asa::render_apsa_in_text(doc, language),
        CitationStyle::Asa6th7th => templates::apsa_asa::render_asa_in_text(doc, language),
        CitationStyle::Chicago18AuthorDate => {
            templates::chicago::render_author_date_in_text(doc, language)
        }
        CitationStyle::Chicago18NotesBib => {
            templates::chicago::render_notes_bib_in_text(doc, language)
        }
        CitationStyle::Chicago18ShortenedNotesBib => {
            templates::chicago::render_shortened_notes_in_text(doc, language)
        }
        CitationStyle::CiteThemRight12thHarvard => {
            templates::harvard::render_ctr_harvard_in_text(doc, language)
        }
        CitationStyle::ElsevierHarvardWithTitles => {
            templates::harvard::render_elsevier_harvard_in_text(doc, language)
        }
        CitationStyle::IeeeV11_29_2023 => templates::ieee::render_in_text(doc, language),
        CitationStyle::Mhra4thNotes => templates::mhra::render_in_text(doc, language),
        CitationStyle::Mla9thInText => templates::mla::render_in_text(doc, language),
        CitationStyle::Nature => templates::nature::render_in_text(doc, language),
        CitationStyle::NlmVancouverCitingMedicine2nd => {
            templates::vancouver::render_in_text(doc, language)
        }
    };
    Ok(result)
}
