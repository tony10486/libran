use once_cell::sync::Lazy;
use regex::Regex;

pub struct PaperIdentifiers {
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
}

static DOI_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+[A-Z0-9]").unwrap());

static ARXIV_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:arxiv:)?(\d{4}\.\d{4,5}(?:v\d+)?)\b|\b(?:arxiv:)?([a-z\-]+(?:\.[a-z]+)?/\d{7}(?:v\d+)?)\b",
    )
    .unwrap()
});

/// Limit identifier search to the first N pages to reduce false matches.
fn first_pages(text: &str, n: usize) -> String {
    text.split('\x0c').take(n).collect::<Vec<_>>().join("\x0c")
}

/// Validate an arXiv new-scheme id (YYMM.NNNNN): year >= 1991, month 01-12.
fn is_valid_arxiv_new_scheme(id: &str) -> bool {
    let core = id.split('v').next().unwrap_or(id);
    let mut parts = core.splitn(2, '.');
    let yy_mm = parts.next().unwrap_or("");
    if yy_mm.len() != 4 || !yy_mm.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let yy: u32 = yy_mm[..2].parse().unwrap_or(0);
    let mm: u32 = yy_mm[2..].parse().unwrap_or(0);
    // arXiv started in 1991; accept 91-99 (1991-1999) and 00-30 (2000-2030).
    let year_ok = (91..=99).contains(&yy) || yy <= 30;
    let month_ok = (1..=12).contains(&mm);
    year_ok && month_ok
}

// Reject matches embedded in URL/path fragments like "pa.2011.0680.DC1.html"
// by checking that the match is not preceded by a letter + dot.
fn is_standalone_arxiv_id(text: &str, match_start: usize) -> bool {
    if match_start == 0 {
        return true;
    }
    let before = &text[..match_start];
    if before.to_lowercase().ends_with("arxiv:") {
        return true;
    }
    let prev = before.chars().last().unwrap();
    if prev.is_whitespace() {
        return true;
    }
    if prev == '.' {
        if before.len() >= 2 {
            let before_dot = before[..before.len() - 1].chars().last().unwrap_or(' ');
            if before_dot.is_alphabetic() {
                return false;
            }
        }
        return true;
    }
    !prev.is_alphanumeric()
}

pub fn search_academic_identifiers(page_text: &str) -> PaperIdentifiers {
    let scope = first_pages(page_text, 2);

    let doi_captures: Vec<regex::Captures> = DOI_RE.captures_iter(&scope).collect();
    let doi_spans: Vec<(usize, usize)> = doi_captures
        .iter()
        .filter_map(|c| c.get(0).map(|m| (m.start(), m.end())))
        .collect();

    let found_doi = doi_captures.first().map(|cap| {
        let mut doi_str = cap[0].to_string().to_lowercase();
        if doi_str.ends_with('.') || doi_str.ends_with(';') || doi_str.ends_with(',') {
            doi_str.pop();
        }
        doi_str
    });

    let found_arxiv = ARXIV_RE.captures_iter(&scope).find_map(|cap| {
        if let Some(new_scheme) = cap.get(1) {
            let overlaps = doi_spans
                .iter()
                .any(|(ds, de)| new_scheme.start() < *de && new_scheme.end() > *ds);
            if overlaps {
                return None;
            }
            if !is_valid_arxiv_new_scheme(new_scheme.as_str()) {
                return None;
            }
            if !is_standalone_arxiv_id(&scope, new_scheme.start()) {
                return None;
            }
            return Some(new_scheme.as_str().to_string());
        }
        if let Some(old_scheme) = cap.get(2) {
            return Some(old_scheme.as_str().to_string());
        }
        None
    });

    PaperIdentifiers {
        doi: found_doi,
        arxiv_id: found_arxiv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doi_extraction() {
        let text = "This work is published at doi:10.1000/xyz123 and referenced elsewhere.";
        let ids = search_academic_identifiers(text);
        assert!(ids.doi.is_some());
        assert!(ids.doi.as_ref().unwrap().contains("10.1000/xyz123"));
    }

    #[test]
    fn test_doi_with_punctuation_stripped() {
        let text = "See 10.1038/nature12373. for details.";
        let ids = search_academic_identifiers(text);
        assert!(ids.doi.is_some());
        let doi = ids.doi.unwrap();
        assert!(!doi.ends_with('.'));
    }

    #[test]
    fn test_arxiv_new_scheme() {
        let text = "arXiv:2301.00123v2 was submitted last week.";
        let ids = search_academic_identifiers(text);
        assert!(ids.arxiv_id.is_some());
        assert_eq!(ids.arxiv_id.as_ref().unwrap(), "2301.00123v2");
    }

    #[test]
    fn test_arxiv_old_scheme() {
        let text = "Preprint hep-th/9901001 is available.";
        let ids = search_academic_identifiers(text);
        assert!(ids.arxiv_id.is_some());
        assert_eq!(ids.arxiv_id.as_ref().unwrap(), "hep-th/9901001");
    }

    #[test]
    fn test_no_identifiers() {
        let text = "This is a plain text without any identifiers.";
        let ids = search_academic_identifiers(text);
        assert!(ids.doi.is_none());
        assert!(ids.arxiv_id.is_none());
    }

    #[test]
    fn test_arxiv_not_matched_inside_doi_suffix() {
        let text = "See doi:10.1098/rspa.2011.0680 for details.";
        let ids = search_academic_identifiers(text);
        assert!(
            ids.arxiv_id.is_none(),
            "should not extract arXiv from DOI suffix: {:?}",
            ids.arxiv_id
        );
        assert!(ids.doi.is_some());
    }

    #[test]
    fn test_arxiv_not_matched_inside_doi_suffix_invalid_month() {
        let text = "doi:10.1006/jcta.2000.3094 is the reference.";
        let ids = search_academic_identifiers(text);
        assert!(
            ids.arxiv_id.is_none(),
            "should not extract arXiv from DOI suffix: {:?}",
            ids.arxiv_id
        );
    }

    #[test]
    fn test_arxiv_new_scheme_valid_month() {
        let text = "arXiv:0711.0189v1 [cs.DS] 1 Nov 2007";
        let ids = search_academic_identifiers(text);
        assert!(ids.arxiv_id.is_some());
        assert_eq!(ids.arxiv_id.as_ref().unwrap(), "0711.0189v1");
    }

    #[test]
    fn test_arxiv_new_scheme_invalid_month_rejected() {
        let text = "Reference 2099.0099 here.";
        let ids = search_academic_identifiers(text);
        assert!(
            ids.arxiv_id.is_none(),
            "month 00 should be rejected: {:?}",
            ids.arxiv_id
        );
    }

    #[test]
    fn test_identifier_search_limited_to_first_pages() {
        // A DOI appearing only on page 3 should not be found.
        let text = "page one text\n\x0cpage two text\n\x0cdoi:10.1234/only.on.page.three";
        let ids = search_academic_identifiers(text);
        assert!(
            ids.doi.is_none(),
            "should not search past first 2 pages: {:?}",
            ids.doi
        );
    }
}
