use once_cell::sync::Lazy;
use regex::Regex;

pub struct PaperIdentifiers {
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
}

static DOI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+[A-Z0-9]").unwrap()
});

static ARXIV_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:arxiv:)?(\d{4}\.\d{4,5}(?:v\d+)?)\b|\b(?:arxiv:)?([a-z\-]+(?:\.[a-z]+)?/\d{7}(?:v\d+)?)\b",
    )
    .unwrap()
});

pub fn search_academic_identifiers(page_text: &str) -> PaperIdentifiers {
    let found_doi = DOI_RE.captures(page_text).map(|cap| {
        let mut doi_str = cap[0].to_string().to_lowercase();
        if doi_str.ends_with('.') || doi_str.ends_with(';') || doi_str.ends_with(',') {
            doi_str.pop();
        }
        doi_str
    });

    let found_arxiv = ARXIV_RE.captures(page_text).map(|cap| {
        if let Some(new_scheme) = cap.get(1) {
            new_scheme.as_str().to_string()
        } else if let Some(old_scheme) = cap.get(2) {
            old_scheme.as_str().to_string()
        } else {
            String::new()
        }
    }).filter(|s| !s.is_empty());

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
}
