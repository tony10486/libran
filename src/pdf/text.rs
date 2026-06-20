use anyhow::Result;
use std::path::Path;

pub fn extract_text(path: &Path) -> Result<String> {
    let doc = unpdf::parse_file(path)?;
    let text = doc.plain_text();
    Ok(text)
}
