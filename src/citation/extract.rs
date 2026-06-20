use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Debug, Default)]
pub struct ReferenceEntry {
    pub raw_text: String,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub pub_year: Option<i64>,
    pub journal: Option<String>,
}

static DOI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"10\.\d{4,9}/[^\s,;]+").expect("DOI regex compile")
});

static ARXIV_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:arXiv:)?(\d{4}\.\d{4,5}(?:v\d+)?)").expect("arXiv regex compile")
});

static YEAR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(19|20)\d{2}\b").expect("year regex compile")
});

static REF_SECTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?is)^(?:references|bibliography|참고문헌|literaturverzeichnis)\s*$")
        .expect("ref section regex compile")
});

static NUM_BRACKET_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*\[(\d+)\]").expect("numbered bracket regex compile")
});

static NUM_DOT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(\d+)\.\s+").expect("numbered dot regex compile")
});

static TITLE_QUOTED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[\""\u{201c}\u{201d}]([^\""\u{201c}\u{201d}]+)[\""\u{201c}\u{201d}]"#)
        .expect("quoted title regex compile")
});

pub fn detect_reference_section(text: &str) -> Option<&str> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|line| REF_SECTION_RE.is_match(line))?;
    let body_start = start + 1;
    if body_start >= lines.len() {
        return None;
    }
    let end = lines[body_start..]
        .iter()
        .position(|line| {
            line.trim().is_empty()
                && lines
                    .get(body_start..)
                    .map_or(0, |s| s.iter().take_while(|l| l.trim().is_empty()).count())
                    > 3
        })
        .map_or(lines.len(), |p| body_start + p);
    Some(lines[body_start..end].join("\n").leak() as &str)
}

pub fn split_reference_entries(section_text: &str) -> Vec<String> {
    let lines: Vec<&str> = section_text.lines().collect();
    let mut entries: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_new_entry = NUM_BRACKET_RE.is_match(trimmed)
            || NUM_DOT_RE.is_match(trimmed);

        if is_new_entry && !current.is_empty() {
            entries.push(current.trim().to_string());
            current.clear();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(trimmed);
    }

    if !current.trim().is_empty() {
        entries.push(current.trim().to_string());
    }

    entries
}

pub fn parse_single_reference(text: &str) -> ReferenceEntry {
    let doi = DOI_RE
        .find(text)
        .map(|m| m.as_str().trim_end_matches(|c: char| c == '.' || c == ',').to_string());

    let arxiv_id = ARXIV_RE
        .captures(text)
        .map(|caps| caps[1].to_string());

    let title = TITLE_QUOTED_RE
        .captures(text)
        .map(|caps| caps[1].trim().to_string());

    let pub_year = YEAR_RE
        .captures(text)
        .and_then(|caps| caps[0].parse::<i64>().ok())
        .filter(|&y| (1900..=2030).contains(&y));

    ReferenceEntry {
        raw_text: text.to_string(),
        doi,
        arxiv_id,
        title,
        authors: None,
        pub_year,
        journal: None,
    }
}

pub fn extract_references(text: &str) -> Vec<ReferenceEntry> {
    let section = match detect_reference_section(text) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let entries = split_reference_entries(section);
    entries.iter().map(|e| parse_single_reference(e)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doi() {
        let entry = parse_single_reference("Smith et al., 10.1234/test.2023, Nature, 2023.");
        assert_eq!(entry.doi.as_deref(), Some("10.1234/test.2023"));
    }

    #[test]
    fn test_parse_arxiv_id() {
        let entry = parse_single_reference("arXiv:2301.01234v2, 2023.");
        assert_eq!(entry.arxiv_id.as_deref(), Some("2301.01234v2"));
    }

    #[test]
    fn test_parse_quoted_title() {
        let entry = parse_single_reference("Smith, \u{201c}Deep Learning\u{201d}, Nature, 2023.");
        assert_eq!(entry.title.as_deref(), Some("Deep Learning"));
    }

    #[test]
    fn test_parse_year() {
        let entry = parse_single_reference("Smith, Deep Learning, Nature, 2023.");
        assert_eq!(entry.pub_year, Some(2023));
    }

    #[test]
    fn test_detect_reference_section() {
        let text = "Introduction...\n\nReferences\n\n[1] Smith, Test, 2023.\n[2] Lee, Demo, 2021.";
        let section = detect_reference_section(text);
        assert!(section.is_some());
        assert!(section.unwrap().contains("Smith"));
    }

    #[test]
    fn test_split_numbered_entries() {
        let section = "[1] First reference.\n[2] Second reference.\n[3] Third one.";
        let entries = split_reference_entries(section);
        assert_eq!(entries.len(), 3);
        assert!(entries[0].starts_with("[1]"));
    }

    #[test]
    fn test_extract_references_end_to_end() {
        let text = "Body text.\n\nReferences\n\n[1] Smith, \"Neural Nets\", 10.1234/nn, 2023.\n[2] Lee, arXiv:2301.05678, 2021.";
        let refs = extract_references(text);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].doi.as_deref(), Some("10.1234/nn"));
        assert_eq!(refs[1].arxiv_id.as_deref(), Some("2301.05678"));
    }
}
