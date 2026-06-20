use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn compute_file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

pub fn copy_to_library(source: &Path, library_dir: &Path, filename: &str) -> Result<PathBuf> {
    if !library_dir.exists() {
        fs::create_dir_all(library_dir)?;
    }
    let dest = library_dir.join(filename);
    fs::copy(source, &dest)?;
    Ok(dest)
}

pub fn build_library_filename(citation_key: &str, extension: &str) -> String {
    format!("{}.{}", citation_key, extension)
}

pub fn check_file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}
