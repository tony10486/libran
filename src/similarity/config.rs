use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// User-editable similarity scoring parameters.
/// Stored as a TOML file at `~/.libran/similarity.toml`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimilarityConfig {
    pub udc_leaf_match: f64,
    pub udc_parent_match: f64,
    pub udc_grandparent_match: f64,
    pub tag_match: f64,
    pub cited_by: f64,
    pub mutual_citation: f64,
    pub year_proximity_max: f64,
    pub year_proximity_scale: f64,
    pub same_conference: f64,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            udc_leaf_match: 5.0,
            udc_parent_match: 2.5,
            udc_grandparent_match: 0.1,
            tag_match: 7.0,
            cited_by: 10.0,
            mutual_citation: 20.0,
            year_proximity_max: 1.0,
            year_proximity_scale: 50.0,
            same_conference: 1.0,
        }
    }
}

impl SimilarityConfig {
    /// Path: ~/.libran/similarity.toml
    pub fn path() -> PathBuf {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".libran").join("similarity.toml")
    }

    /// Load from TOML file. Creates default file if missing.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match toml::from_str::<SimilarityConfig>(&content) {
                        Ok(cfg) => return cfg,
                        Err(e) => {
                            eprintln!("similarity.toml parse error: {e}, using defaults");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("similarity.toml read error: {e}, using defaults");
                }
            }
        }
        let cfg = SimilarityConfig::default();
        if let Err(e) = cfg.save() {
            eprintln!("Failed to write default similarity.toml: {e}");
        }
        cfg
    }

    /// Save config to TOML file.
    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut file = fs::File::create(&path)?;
        file.write_all(toml_str.as_bytes())?;
        Ok(())
    }
}
