use once_cell::sync::Lazy;
use regex::Regex;

static JOURNAL_PREFIXES: &[&str] = &[
    "IEEE TRANSACTIONS ON",
    "IEEE JOURNAL ON",
    "JOURNAL OF",
    "PROCEEDINGS OF",
    "LECTURE NOTES IN",
    "ACM TRANSACTIONS ON",
    "PHYSICAL REVIEW LETTERS",
    "PHYSICAL REVIEW",
    "PROC. R. SOC.",
    "LINEAR ALGEBRA AND ITS APPLICATIONS",
    "NATURE",
    "SCIENCE",
    "SOCIAL NETWORKS",
    "J. MATH. PHYS.",
    "PHASE TRANSITIONS",
    "DISCRETE MATHEMATICS",
    "DISCRETE MATH.",
    "DISCRETE MATH",
];

static YEAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(19[5-9]\d|20[0-3]\d)\b").unwrap());

// Trailing noise markers that end a journal name line:
//  - ", VOL. 26" (volume number)
//  - ", 439 (2013)" (comma + volume + year)
//  - " 2016.08:null" (volume.issue:page with null)
//  - " 10.1016/" (DOI)
//  - ", 2016" trailing year
static JOURNAL_NOISE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i),?\s+(?:vol\.|no\.)\s*\d|,?\s+\d{1,4}\s*\(|\s+\d{4}\.\d{2}\s*:|\s+10\.\d{4,}/|,?\s+\d{4}\s*$").unwrap()
});

static DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bdownload(?:ed)?\b").unwrap()
});

static DOI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)10\.\d{4,9}/[-._;()/:A-Z0-9]+[A-Z0-9]").unwrap()
});

pub fn extract_journal_and_year(text: &str) -> (Option<String>, Option<i64>) {
    let first_page = text.split('\x0c').next().unwrap_or("");
    let first_10_pages: String = text.split('\x0c').take(10).collect::<Vec<_>>().join("\x0c");

    let journal = JOURNAL_PREFIXES
        .iter()
        .find_map(|prefix| {
            for line in first_page.lines().take(10) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let upper = trimmed.to_uppercase();
                if upper.starts_with(prefix) {
                    let name = clean_journal_name(trimmed);
                    if name.to_uppercase().len() >= prefix.len() {
                        return Some(name);
                    }
                }
            }
            None
        });

    let year = extract_year(&first_10_pages);

    (journal, year)
}

fn clean_journal_name(raw: &str) -> String {
    let mut name = raw.trim();
    if let Some(m) = JOURNAL_NOISE_RE.find(name) {
        name = &name[..m.start()];
    }
    name.trim_end_matches(',').trim().to_string()
}

fn extract_year(first_page: &str) -> Option<i64> {
    for line in first_page.lines() {
        let trimmed = line.trim();
        if trimmed.to_lowercase().contains("doi")
            || trimmed.contains("10.")
            || JOURNAL_PREFIXES
                .iter()
                .any(|p| trimmed.to_uppercase().contains(p))
        {
            if let Some(y) = year_from_line_excluding_doi(trimmed) {
                return Some(y);
            }
        }
    }
    for line in first_page.lines() {
        let trimmed = line.trim();
        if DOWNLOAD_RE.is_match(trimmed) {
            continue;
        }
        if let Some(y) = year_from_line_excluding_doi(trimmed) {
            return Some(y);
        }
    }
    None
}

pub fn has_year_pattern(line: &str) -> bool {
    year_from_line_excluding_doi(line).is_some()
}

fn year_from_line_excluding_doi(line: &str) -> Option<i64> {
    let cleaned = DOI_RE.replace_all(line, "");
    YEAR_RE
        .captures(&cleaned)
        .and_then(|cap| cap[1].parse::<i64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_journal_from_ieee_header() {
        let text = "IEEE TRANSACTIONS ON PATTERN ANALYSIS AND MACHINE INTELLIGENCE, VOL. 26, NO. 9, SEPTEMBER 2004 1243\n\nDistance-Preserving Projection...\n";
        let (journal, year) = extract_journal_and_year(text);
        assert_eq!(
            journal.as_deref(),
            Some("IEEE TRANSACTIONS ON PATTERN ANALYSIS AND MACHINE INTELLIGENCE")
        );
        assert_eq!(year, Some(2004));
    }

    #[test]
    fn test_extract_journal_from_lecture_notes() {
        let text = "Lecture Notes in Computer Science, vol. 1234, 2007\nSome Title\n";
        let (journal, year) = extract_journal_and_year(text);
        assert!(journal.is_some());
        assert_eq!(year, Some(2007));
    }

    #[test]
    fn test_extract_year_from_text() {
        let text = "Some conference proceedings, 2015\nPaper title here\n";
        let (_journal, year) = extract_journal_and_year(text);
        assert_eq!(year, Some(2015));
    }

    #[test]
    fn test_clean_journal_name_strips_volume_null() {
        let cleaned = clean_journal_name("Discrete Math. Algorithm. Appl. 2016.08:null-null");
        assert_eq!(cleaned, "Discrete Math. Algorithm. Appl.");
    }

    #[test]
    fn test_clean_journal_name_strips_volume_pages_doi() {
        let cleaned = clean_journal_name("Linear Algebra and its Applications, 439 (2013) 3038–3043. 10.1016/j.laa.2013.08.039");
        assert_eq!(cleaned, "Linear Algebra and its Applications");
    }

    #[test]
    fn test_extract_year_skips_download_header() {
        let text = "Downloaded 16 June 2016\nSome Title\nPublished in Journal, 1993\n";
        let (_journal, year) = extract_journal_and_year(text);
        assert_eq!(year, Some(1993));
    }

    #[test]
    fn test_extract_journal_discrete_math() {
        let text = "Discrete Math. Algorithm. Appl. 2016.08:null-null\nSome title\n";
        let (journal, _year) = extract_journal_and_year(text);
        assert_eq!(journal.as_deref(), Some("Discrete Math. Algorithm. Appl."));
    }

    #[test]
    fn test_extract_journal_linear_algebra() {
        let text = "Linear Algebra and its Applications, 439 (2013) 3038–3043. 10.1016/j.laa.2013.08.039\nSome title\n";
        let (journal, year) = extract_journal_and_year(text);
        assert_eq!(journal.as_deref(), Some("Linear Algebra and its Applications"));
        assert_eq!(year, Some(2013));
    }
}
