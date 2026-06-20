use anyhow::Result;
use lopdf::Document as LopdfDocument;
use std::path::Path;

use super::RawMetadata;

pub fn extract_metadata(path: &Path) -> Result<RawMetadata> {
    let doc = LopdfDocument::load(path)?;
    let mut meta = RawMetadata::default();

    if let Ok(info) = doc.trailer.get(b"Info")
        && let Ok(info_ref) = info.as_reference()
            && let Ok(info_obj) = doc.get_object(info_ref)
                && let Ok(dict) = info_obj.as_dict() {
                    meta.title = get_string(dict, b"Title");
                    meta.journal = get_string(dict, b"Subject");
                    if let Some(author_str) = get_string(dict, b"Author") {
                        meta.authors = author_str
                            .split(';')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }

    Ok(meta)
}

fn get_string(dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key).ok().and_then(|obj| {
        if let Ok(s) = obj.as_str() {
            Some(String::from_utf8_lossy(s).to_string())
        } else if let Ok(_ref_obj) = obj.as_reference() {
            None
        } else {
            None
        }
    })
}
