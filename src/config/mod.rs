use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::api::ApiMode;
use crate::citation::CitationKeyMode;
use crate::storage::FileStoragePolicy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_mode: ApiMode,
    pub user_email: Option<String>,
    pub file_storage_policy: FileStoragePolicy,
    pub library_path: PathBuf,
    pub citation_key_mode: CitationKeyModeConfig,
    pub citation_key_template: Option<String>,
    pub primary_scheme: String,
    pub enabled_schemes: Vec<String>,
    pub label_language: String,
    pub db_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CitationKeyModeConfig {
    AuthorYear,
    AuthorYearTitle,
    AuthorYearHash,
    Custom,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let libran_dir = home.join(".libran");

        AppConfig {
            api_mode: ApiMode::FullyOffline,
            user_email: None,
            file_storage_policy: FileStoragePolicy::CopyToLibrary,
            library_path: libran_dir.join("library"),
            citation_key_mode: CitationKeyModeConfig::AuthorYear,
            citation_key_template: None,
            primary_scheme: "udc".to_string(),
            enabled_schemes: vec!["udc".to_string(), "physh".to_string(), "msc".to_string()],
            label_language: "en".to_string(),
            db_path: libran_dir.join("libran.db"),
        }
    }
}

impl AppConfig {
    pub fn to_citation_key_mode(&self) -> CitationKeyMode {
        match self.citation_key_mode {
            CitationKeyModeConfig::AuthorYear => CitationKeyMode::AuthorYear,
            CitationKeyModeConfig::AuthorYearTitle => CitationKeyMode::AuthorYearTitle,
            CitationKeyModeConfig::AuthorYearHash => CitationKeyMode::AuthorYearHash,
            CitationKeyModeConfig::Custom => {
                let template = self.citation_key_template.clone().unwrap_or_default();
                CitationKeyMode::Custom(template)
            }
        }
    }
}
