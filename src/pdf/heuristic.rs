use once_cell::sync::Lazy;
use regex::Regex;

static SECTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d+(\.\d+)*\.?\s+\w").unwrap()
});

const SKIP_MARKERS: &[&str] = &[
    "abstract", "keywords", "introduction", "university", "institute",
    "department", "email", "www.", "http", "@", "arxiv:", "received",
    "accepted", "published", "this article", "this paper", "this work",
    "journal", "proceedings", "conference", "vol.", "volume", "pp.",
    "press", "springer", "elsevier", "ieee", "acm", "doi", "figure",
    "table", "section", "chapter", "max planck", "mit ", "stanford",
    "harvard", "caltech", "berkeley",
];

use crate::pdf::authors::{strip_author_from_title, strip_trailing_name_from_text, AUTHOR_MARKERS};

pub fn guess_title(full_text: &str) -> Option<String> {
    let first_page = extract_first_page(full_text);
    if let Some(title) = guess_title_from_first_page(&first_page) {
        return Some(strip_author_from_title(&title));
    }
    guess_title_from_full_text(full_text).map(|t| strip_author_from_title(&t))
}

fn extract_first_page(text: &str) -> String {
    text.split('\x0c').next().unwrap_or("").to_string()
}

fn guess_title_from_first_page(page: &str) -> Option<String> {
    let lines: Vec<&str> = page.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }

    let abstract_idx = find_abstract_marker(&lines);
    let search_end = abstract_idx.unwrap_or(lines.len().min(15));

    let mut candidates: Vec<(usize, &str)> = Vec::new();
    for (i, line) in lines.iter().enumerate().take(search_end) {
        let trimmed = line.trim();
        if is_title_candidate(trimmed) {
            candidates.push((i, trimmed));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|(idx, line)| {
        let word_count = line.split_whitespace().count();
        let position_score = *idx;
        let length_score = if (4..=15).contains(&word_count) { 0 } else { 50 };
        position_score + length_score
    });

    let (best_idx, best_line) = candidates[0];

    let next_line = lines
        .get(best_idx + 1)
        .map(|s| s.trim())
        .unwrap_or("");
    let has_author_next = AUTHOR_MARKERS.iter().any(|m| next_line.contains(m));

    if has_author_next && best_line.ends_with(',') {
        if let Some(stripped) = strip_trailing_name_from_text(best_line) {
            return Some(stripped);
        }
    }

    Some(best_line.to_string())
}

fn guess_title_from_full_text(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }

    let abstract_idx = find_abstract_marker(&lines);
    let search_end = abstract_idx.unwrap_or(lines.len().min(30));

    let mut candidates: Vec<(usize, &str)> = Vec::new();
    for i in (0..search_end).rev() {
        let trimmed = lines[i].trim();
        if is_title_candidate(trimmed) {
            candidates.push((i, trimmed));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|(idx, line)| {
        let word_count = line.split_whitespace().count();
        let distance_penalty = *idx;
        let length_penalty = if (4..=15).contains(&word_count) { 0 } else { 100 };
        distance_penalty + length_penalty
    });

    Some(candidates[0].1.to_string())
}

fn find_abstract_marker(lines: &[&str]) -> Option<usize> {
    for (i, line) in lines.iter().enumerate().take(80) {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower == "abstract"
            || lower == "abstract."
            || lower.starts_with("abstract—")
            || lower.starts_with("abstract -")
            || lower.starts_with("abstract:")
            || lower.starts_with("abstract.")
            || lower.starts_with("abstract ")
        {
            return Some(i);
        }
    }
    None
}

/// Public wrapper for cross-module use (e.g. `authors::guess_authors`).
pub fn find_abstract_marker_pub(lines: &[&str]) -> Option<usize> {
    find_abstract_marker(lines)
}

fn is_title_candidate(trimmed: &str) -> bool {
    if trimmed.len() < 10 || trimmed.len() > 200 {
        return false;
    }

    let lower = trimmed.to_lowercase();

    if SKIP_MARKERS.iter().any(|m| lower.contains(m)) {
        return false;
    }

    if SECTION_RE.is_match(trimmed) {
        return false;
    }

    if trimmed.chars().filter(|c| c.is_ascii_digit()).count() > trimmed.len() / 3 {
        return false;
    }

    let word_count = trimmed.split_whitespace().count();
    if !(3..=25).contains(&word_count) {
        return false;
    }

    if !trimmed.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        return false;
    }

    if trimmed.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
        return false;
    }

    if trimmed.contains("  ") {
        return false;
    }

    let period_count = trimmed.chars().filter(|c| *c == '.').count();
    if period_count > 1 {
        return false;
    }
    if period_count == 1 && trimmed.ends_with('.') {
        return false;
    }

    if trimmed.matches(',').count() > 1 {
        return false;
    }

    if trimmed.contains(", and ") || trimmed.contains(", or ") || trimmed.contains(", but ") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_title_longest_line() {
        let text = "Short\nThis is a much longer title that should be selected\nNext line\n";
        let result = guess_title(text);
        assert!(result.is_some());
    }

    #[test]
    fn test_guess_title_empty() {
        let result = guess_title("");
        assert!(result.is_none());
    }

    #[test]
    fn test_guess_title_before_abstract() {
        let text = "A Tutorial on Spectral Clustering\n\nAbstract\nThis is the abstract text that goes on and on about spectral clustering.\n";
        let result = guess_title(text);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "A Tutorial on Spectral Clustering");
    }

    #[test]
    fn test_guess_title_multiple_lines() {
        let text = "A Tutorial on\nSpectral Clustering\n\nAbstract\nLong abstract here.\n";
        let result = guess_title(text);
        assert!(result.is_some());
    }

    #[test]
    fn test_guess_title_jumbled_text() {
        let text = "algorithms. It is simple to implement, and very often outperforms traditional clustering algorithms.\n\n1 Introduction\n\nClustering is one of the most widely used techniques for exploratory data analysis.\n\narXiv:0711.0189v1  [cs.DS]  1 Nov 2007\n\nA Tutorial on Spectral Clustering\n\nMax Planck Institute for Biological Cybernetics\n\nUlrike von Luxburg\n\nAbstract\n\nIn recent years, spectral clustering has become one of the most popular modern clustering algorithms.\n";
        let result = guess_title(text);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "A Tutorial on Spectral Clustering");
    }

    // Regression tests from real unpdf output of liyang2004.pdf.
    // unpdf merges multi-column layouts, appending author info to the title line
    // and rendering "Abstract—" as "Abstract —" (space + em-dash).

    #[test]
    fn test_find_abstract_marker_em_dash() {
        let lines = vec!["Some title", "Abstract —A distance-preserving method..."];
        assert_eq!(find_abstract_marker(&lines), Some(1));
    }

    #[test]
    fn test_find_abstract_marker_uppercase() {
        let lines = vec!["Title", "ABSTRACT", "Some text"];
        assert_eq!(find_abstract_marker(&lines), Some(1));
    }

    #[test]
    fn test_find_abstract_marker_colon() {
        let lines = vec!["Title", "Abstract:", "Text"];
        assert_eq!(find_abstract_marker(&lines), Some(1));
    }

    #[test]
    fn test_guess_title_strips_merged_author() {
        let text = "IEEE TRANSACTIONS ON PATTERN ANALYSIS AND MACHINE INTELLIGENCE, VOL. 26, NO. 9, SEPTEMBER 2004 1243\n\nDistance-Preserving Projection of High-Dimensional Data for Nonlinear Dimensionality Reduction Li Yang,\n\nSenior Member , IEEE\n\nAbstract —A distance-preserving method is presented.\n";
        let result = guess_title(text);
        assert!(result.is_some());
        let title = result.unwrap();
        assert!(!title.contains("Li Yang"), "title should not contain author: {title}");
        assert!(title.contains("Distance-Preserving Projection"));
    }

    #[test]
    fn e2e_liyang2004_process_file() {
        let path = std::path::PathBuf::from("tmp/[중요]liyang2004.pdf");
        if !path.exists() {
            eprintln!("SKIP e2e: PDF not found");
            return;
        }
        let meta = super::super::process_file(&path).expect("process_file");
        assert!(
            meta.title.as_deref().is_some_and(|t| t.contains("Distance-Preserving Projection")),
            "title wrong: {meta:?}"
        );
        assert!(
            !meta.title.as_deref().unwrap_or("").contains("Li Yang"),
            "title should not contain author: {meta:?}"
        );
        assert!(
            meta.authors.iter().any(|a| a.contains("Yang")),
            "authors should contain Yang: {meta:?}"
        );
        assert_eq!(meta.pub_year, Some(2004), "pub_year wrong: {meta:?}");
        assert!(
            meta.journal.as_deref().is_some_and(|j| j.contains("IEEE TRANSACTIONS")),
            "journal wrong: {meta:?}"
        );
    }
}
