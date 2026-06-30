pub mod authors;
pub mod bookmarks;
pub mod heuristic;
pub mod identifiers;
pub mod journal;
pub mod metadata;
pub mod okular;
pub mod sioyek;
pub mod text;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RawMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub journal: Option<String>,
    pub pub_year: Option<i64>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub abstract_text: Option<String>,
    pub keywords: Vec<String>,
    pub source: MetadataSource,
    pub body_text: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum MetadataSource {
    #[default]
    PdfExtract,
    Crossref,
    Arxiv,
    Manual,
}

pub trait PdfMetadataExtractor {
    fn extract_metadata(&self, path: &Path) -> Result<RawMetadata>;
}

pub trait PdfTextExtractor {
    fn extract_text(&self, path: &Path) -> Result<String>;
}

pub fn process_file(path: &Path) -> Result<RawMetadata> {
    let mut metadata = RawMetadata {
        source: MetadataSource::PdfExtract,
        ..Default::default()
    };

    if let Ok(meta) = metadata::extract_metadata(path) {
        metadata.title = meta.title;
        metadata.authors = meta.authors;
        metadata.journal = meta.journal;
        metadata.pub_year = meta.pub_year;
    }

    if let Ok(text) = text::extract_text(path) {
        metadata.body_text = Some(text.clone());

        let ids = identifiers::search_academic_identifiers(&text);
        metadata.doi = ids.doi;
        metadata.arxiv_id = ids.arxiv_id;

        if metadata.arxiv_id.is_none() {
            metadata.arxiv_id = extract_arxiv_from_filename(path);
        }

        let has_identifier = metadata.doi.is_some() || metadata.arxiv_id.is_some();

        if !has_identifier {
            if metadata.title.is_none() {
                metadata.title = heuristic::guess_title(&text);
            }

            if metadata.authors.is_empty() {
                metadata.authors = authors::guess_authors(&text);
            }

            if metadata.journal.is_none() || metadata.pub_year.is_none() {
                let (journal, year) = journal::extract_journal_and_year(&text);
                if metadata.journal.is_none() {
                    metadata.journal = journal;
                }
                if metadata.pub_year.is_none() {
                    metadata.pub_year = year;
                }
            }
        }
    }

    if metadata.title.is_none() {
        metadata.title = extract_title_from_filename(path);
    }

    Ok(metadata)
}

fn extract_arxiv_from_filename(path: &Path) -> Option<String> {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static ARXIV_FILE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(\d{4}\.\d{4,5}(?:v\d+)?)").unwrap());

    let filename = path.file_name()?.to_string_lossy();
    let caps = ARXIV_FILE_RE.captures(&filename)?;
    Some(caps[1].to_string())
}

fn extract_title_from_filename(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let cleaned = stem.replace(['_', '-'], " ");
    let title = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() { None } else { Some(title) }
}
