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
        && let Ok(dict) = info_obj.as_dict()
    {
        if let Some(title) = get_string(dict, b"Title") {
            if is_plausible_title(&title) {
                meta.title = Some(title);
            }
        }
        if let Some(journal) = get_string(dict, b"Subject") {
            if is_plausible_metadata(&journal) {
                meta.journal = Some(journal);
            }
        }
        if let Some(author_str) = get_string(dict, b"Author") {
            if is_plausible_metadata(&author_str) {
                meta.authors = parse_metadata_authors(&author_str);
            }
        }
    }

    Ok(meta)
}

fn looks_like_initial_continuation(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return false;
    }
    let non_et: Vec<&&str> = words
        .iter()
        .filter(|w| {
            let lw = w.to_lowercase();
            lw != "et" && lw != "al" && lw != "al."
        })
        .collect();
    if non_et.is_empty() {
        return false;
    }
    non_et.iter().all(|w| {
        let w = w.trim_end_matches('.');
        w.len() <= 2 && w.chars().all(|c| c.is_ascii_uppercase())
    })
}

fn merge_initial_continuations(parts: &[String]) -> Vec<String> {
    let mut merged: Vec<String> = Vec::new();
    for part in parts {
        if looks_like_initial_continuation(part) {
            if let Some(last) = merged.last_mut() {
                last.push_str(", ");
                last.push_str(part);
            } else {
                merged.push(part.clone());
            }
        } else {
            merged.push(part.clone());
        }
    }
    merged
}

fn parse_metadata_authors(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let semicolon_segs: Vec<&str> = trimmed
        .split(';')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect();
    if semicolon_segs.len() > 1 {
        return semicolon_segs.iter().map(|x| x.to_string()).collect();
    }

    let single = semicolon_segs.first().copied().unwrap_or(trimmed);
    let lower = single.to_lowercase();
    if let Some(pos) = lower.find(" and ") {
        let mut result: Vec<String> = Vec::new();
        let rest = &single[pos + 5..];
        let before = single[..pos].trim();
        if !before.is_empty() {
            let raw: Vec<String> = before
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            result.extend(merge_initial_continuations(&raw));
        }
        let rest_lower = rest.to_lowercase();
        if let Some(pos2) = rest_lower.find(" and ") {
            let mid = rest[..pos2].trim();
            let after = rest[pos2 + 5..].trim();
            if !mid.is_empty() {
                result.push(mid.to_string());
            }
            if !after.is_empty() {
                result.push(after.to_string());
            }
        } else if !rest.is_empty() {
            let raw: Vec<String> = rest
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            result.extend(merge_initial_continuations(&raw));
        }
        if !result.is_empty() {
            return result;
        }
    }

    if single.contains(',') {
        let parts: Vec<String> = single
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect();
        if parts.len() >= 2
            && parts
                .iter()
                .all(|p| p.split_whitespace().count() <= 4 && p.len() <= 40)
        {
            let merged = merge_initial_continuations(&parts);
            return merged;
        }
    }

    vec![single.to_string()]
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

fn is_plausible_title(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_lowercase();
    if lower == "untitled" || lower == "untitled document" {
        return false;
    }
    if lower.starts_with("doi:") || lower.starts_with("doi ") {
        return false;
    }
    is_plausible_metadata(trimmed)
}

fn is_plausible_metadata(s: &str) -> bool {
    if s.contains('\0') || s.contains('\t') {
        return false;
    }
    if s.contains(":null") || s.contains("-null") {
        return false;
    }
    if has_doi_pattern(s) {
        return false;
    }
    let printable = s.chars().filter(|c| !c.is_control()).count();
    if s.is_empty() || printable == 0 {
        return false;
    }
    printable * 4 >= s.chars().count() * 3
}

fn has_doi_pattern(s: &str) -> bool {
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find("10.") {
        let rest = &lower[pos + 3..];
        let digit_count = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count >= 4 {
            if let Some(slash_pos) = rest.find('/') {
                return slash_pos <= digit_count;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_plausible_title_rejects_untitled() {
        assert!(!is_plausible_title("untitled"));
        assert!(!is_plausible_title("Untitled"));
        assert!(!is_plausible_title(""));
        assert!(!is_plausible_title("   "));
    }

    #[test]
    fn test_is_plausible_title_rejects_doi() {
        assert!(!is_plausible_title("doi:10.1016/j.socnet.2004.11.009"));
        assert!(!is_plausible_title("DOI:10.1016/j.socnet.2004.11.009"));
    }

    #[test]
    fn test_is_plausible_title_accepts_real() {
        assert!(is_plausible_title("A Real Paper Title"));
    }

    #[test]
    fn test_is_plausible_metadata_rejects_null_bytes() {
        assert!(!is_plausible_metadata("\0D\0e\0s\0c\0o\0n\0o\0c\0i\0d\0o"));
    }

    #[test]
    fn test_is_plausible_metadata_rejects_tabs() {
        assert!(!is_plausible_metadata(
            "Subject collections\tArticles on similar topics"
        ));
    }

    #[test]
    fn test_is_plausible_metadata_rejects_null_pattern() {
        assert!(!is_plausible_metadata(
            "Discrete Math. Algorithm. Appl. 2016.08:null-null"
        ));
    }

    #[test]
    fn test_is_plausible_metadata_rejects_doi_in_journal() {
        assert!(!is_plausible_metadata(
            "Linear Algebra and its Applications, 439 (2013) 3038–3043. 10.1016/j.laa.2013.08.039"
        ));
    }

    #[test]
    fn test_is_plausible_metadata_accepts_normal() {
        assert!(is_plausible_metadata("Discrete Mathematics"));
    }
}
