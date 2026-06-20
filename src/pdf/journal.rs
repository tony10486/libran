use once_cell::sync::Lazy;
use regex::Regex;

static JOURNAL_PREFIXES: &[&str] = &[
    "IEEE TRANSACTIONS ON",
    "IEEE JOURNAL ON",
    "JOURNAL OF",
    "PROCEEDINGS OF",
    "LECTURE NOTES IN",
    "ACM TRANSACTIONS ON",
    "PHYSICAL REVIEW",
    "NATURE",
    "SCIENCE",
];

static YEAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(19[5-9]\d|20[0-3]\d)\b").unwrap());

pub fn extract_journal_and_year(text: &str) -> (Option<String>, Option<i64>) {
    let first_page = text.split('\x0c').next().unwrap_or("");
    let first_line = first_page.lines().next().unwrap_or("").trim();

    let journal = JOURNAL_PREFIXES
        .iter()
        .find_map(|prefix| {
            let upper = first_line.to_uppercase();
            if upper.starts_with(prefix) {
                let end = first_line.find(',').unwrap_or(first_line.len());
                let name = first_line[..end].trim();
                if name.len() > prefix.len() {
                    Some(name.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        });

    let year = YEAR_RE
        .captures(first_line)
        .or_else(|| YEAR_RE.captures(first_page))
        .and_then(|cap| cap[1].parse::<i64>().ok());

    (journal, year)
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
}
