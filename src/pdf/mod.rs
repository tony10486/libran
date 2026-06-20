pub mod heuristic;
pub mod identifiers;
pub mod metadata;
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
        let first_pages = extract_first_pages(&text, 2);
        let ids = identifiers::search_academic_identifiers(&first_pages);
        metadata.doi = ids.doi;
        metadata.arxiv_id = ids.arxiv_id;

        if metadata.title.is_none() {
            metadata.title = heuristic::guess_title(&first_pages);
        }
    }

    Ok(metadata)
}

fn extract_first_pages(text: &str, page_count: usize) -> String {
    let pages: Vec<&str> = text.split('\x0c').collect();
    pages.into_iter().take(page_count).collect::<Vec<_>>().join("\n")
}
