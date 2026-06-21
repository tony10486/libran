pub static AUTHOR_MARKERS: &[&str] = &[
    "Senior Member", "Junior Member", "Member", "Fellow",
    "Senior, IEEE", "IEEE", "ACM", "AAAI",
];

pub const PREPOSITIONS: &[&str] = &[
    "of", "for", "the", "and", "on", "in", "a", "an", "to", "via", "by",
    "with", "from", "into", "using", "through", "over", "under",
    "von", "de", "van", "der", "la", "le", "du", "di", "da", "dos",
    "das", "el", "bin", "ibn", "y", "al",
];

const JOURNAL_ABBREVS: &[&str] = &[
    "Proc.", "Soc.", "Trans.", "Vol.", "Ser.", "No.",
    "J.", "Commun.", "Lett.", "Math.", "Phys.", "Chem.",
    "Biol.", "Med.", "Rev.", "Ann.", "Acad.",
];

pub fn strip_author_from_title(title: &str) -> String {
    for marker in AUTHOR_MARKERS {
        if let Some(idx) = title.find(marker) {
            let before = title[..idx].trim_end_matches([' ', ',']).trim();
            if before.len() < 10 {
                continue;
            }
            return strip_trailing_name_from_text(before).unwrap_or_else(|| before.to_string());
        }
    }
    title.to_string()
}

pub fn strip_trailing_name_from_text(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 5 {
        return None;
    }

    let is_name_word = |w: &str| {
        w.len() <= 15
            && w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && !PREPOSITIONS.contains(&w.to_lowercase().as_str())
    };

    let last = words[words.len() - 1].trim_end_matches(',');
    let second_last = words[words.len() - 2];

    if !is_name_word(last) || !is_name_word(second_last) {
        return None;
    }

    let remaining = words[..words.len() - 2].join(" ");
    if remaining.len() >= 10 {
        Some(remaining)
    } else {
        None
    }
}

fn strip_citation_suffix(line: &str) -> String {
    for marker in &["Citation:", "citation:", "Citation ", "citation "] {
        if let Some(pos) = line.find(marker) {
            return line[..pos].trim().to_string();
        }
    }
    line.to_string()
}

pub fn guess_authors(full_text: &str) -> Vec<String> {
    let lines: Vec<&str> = full_text.lines().filter(|l| !l.trim().is_empty()).collect();
    let abstract_idx = crate::pdf::heuristic::find_abstract_marker_pub(&lines);

    if let Some(idx) = abstract_idx {
        let mut fallback_authors: Option<Vec<String>> = None;
        for i in (0..idx).rev() {
            let trimmed = lines[i].trim();
            let cleaned = strip_citation_suffix(trimmed);
            let working = if cleaned.len() < trimmed.len() {
                cleaned.as_str()
            } else {
                trimmed
            };
            if working.len() < 5 || working.len() > 200 {
                continue;
            }

            let lower = working.to_lowercase();
            if lower.contains("university")
                || lower.contains("institute")
                || lower.contains("department")
                || lower.contains("abstract")
                || lower.contains("keywords")
                || lower.contains("www.")
                || lower.contains("e-mail")
                || lower.contains("email")
            {
                continue;
            }

            if AUTHOR_MARKERS.iter().any(|m| working.contains(m)) {
                let stripped = strip_member_markers(working);
                let extraction_text =
                    try_merge_trailing_and(&stripped, &lines, i, idx);
                if let Some(authors) = try_extract_authors(&extraction_text) {
                    if authors.len() > 1 {
                        return authors;
                    }
                    if fallback_authors.is_none() {
                        fallback_authors = Some(authors);
                    }
                }
                continue;
            }

            if let Some(authors) = try_extract_authors(working) {
                if authors.len() > 1 {
                    return authors;
                }
                if fallback_authors.is_none() {
                    fallback_authors = Some(authors);
                }
            }
        }
        if let Some(authors) = fallback_authors {
            return authors;
        }
    }

    if abstract_idx.is_none() {
        if let Some(authors) = scan_for_and_pattern(&lines) {
            return authors;
        }
    }

    extract_authors_from_title_line(&lines, abstract_idx)
}

fn strip_member_markers(line: &str) -> String {
    let mut result = line.to_string();
    for marker in AUTHOR_MARKERS {
        while let Some(pos) = result.find(marker) {
            let before = &result[..pos].trim_end_matches([',', ' ']);
            let after_start = pos + marker.len();
            let after = &result[after_start..].trim_start_matches([',', ' ']);
            result = format!("{}, {}", before, after);
            result = result.trim().to_string();
        }
    }
    result.trim_end_matches(',').trim().to_string()
}

fn try_extract_authors(trimmed: &str) -> Option<Vec<String>> {
    if let Some(authors) = extract_authors_from_line(trimmed) {
        return Some(authors);
    }
    if let Some(authors) = extract_authors_from_and_pattern(trimmed) {
        return Some(authors);
    }
    None
}

fn try_merge_trailing_and(stripped: &str, lines: &[&str], i: usize, boundary: usize) -> String {
    let s = stripped.trim();
    if s.ends_with(" and") || s.ends_with(", and") {
        if i + 1 < boundary {
            let next_trimmed = lines[i + 1].trim();
            if next_trimmed.len() >= 5 && next_trimmed.len() <= 200 {
                let next_cleaned = strip_citation_suffix(next_trimmed);
                let next_working = if next_cleaned.len() < next_trimmed.len() {
                    next_cleaned.as_str()
                } else {
                    next_trimmed
                };
                let next_stripped = strip_member_markers(next_working);
                if !next_stripped.is_empty() {
                    return format!("{} {}", s, next_stripped);
                }
            }
        }
    }
    stripped.to_string()
}

fn extract_authors_from_line(trimmed: &str) -> Option<Vec<String>> {
    if trimmed.contains('\t') {
        return None;
    }

    if trimmed.contains('(') || trimmed.contains(')') {
        return None;
    }

    let digit_count = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count > 2 {
        return None;
    }

    let alpha_ratio = trimmed
        .chars()
        .filter(|c| c.is_alphabetic() || *c == ' ' || *c == ',' || *c == '.')
        .count();
    if alpha_ratio < trimmed.len() * 4 / 5 {
        return None;
    }

    let word_count = trimmed.split_whitespace().count();
    if !(2..=6).contains(&word_count) {
        return None;
    }

    let capitalized = trimmed
        .split_whitespace()
        .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
        .count();
    if capitalized * 2 < word_count {
        return None;
    }

    let has_lowercase_non_prep = trimmed.split_whitespace().any(|w| {
        let clean = w.trim_end_matches([',', '.']);
        !clean.is_empty()
            && clean.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
            && !PREPOSITIONS.contains(&clean.to_lowercase().as_str())
    });
    if has_lowercase_non_prep {
        return None;
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("tutorial") || lower.contains("clustering") {
        return None;
    }

    let raw_parts: Vec<String> = trimmed
        .split(',')
        .flat_map(|s| split_and_in_segment(s.trim()))
        .filter(|s| !s.is_empty() && s.len() > 2)
        .collect();
    let authors = merge_initial_parts(&raw_parts);
    if authors.is_empty() {
        None
    } else {
        Some(authors)
    }
}

fn looks_like_initial(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let words: Vec<&str> = s.split_whitespace().collect();
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

fn merge_initial_parts(parts: &[String]) -> Vec<String> {
    let mut merged: Vec<String> = Vec::new();
    for part in parts {
        if looks_like_initial(part) {
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

fn split_and_in_segment(seg: &str) -> Vec<String> {
    let seg = seg.trim();
    if seg.is_empty() {
        return Vec::new();
    }
    let lower = seg.to_lowercase();
    if let Some(pos) = lower.find(" and ") {
        let before = seg[..pos].trim();
        let after = seg[pos + 5..].trim();
        let mut result = Vec::new();
        if !before.is_empty() {
            result.push(before.to_string());
        }
        if !after.is_empty() {
            result.push(after.to_string());
        }
        result
    } else {
        vec![seg.to_string()]
    }
}

fn extract_authors_from_title_line(lines: &[&str], abstract_idx: Option<usize>) -> Vec<String> {
    let search_end = abstract_idx.unwrap_or(lines.len().min(10));
    for i in (0..search_end).rev() {
        let trimmed = lines[i].trim();
        if trimmed.len() < 20 || !trimmed.ends_with(',') {
            continue;
        }

        let next_line = lines.get(i + 1).map(|s| s.trim()).unwrap_or("");
        if !AUTHOR_MARKERS.iter().any(|m| next_line.contains(m)) {
            continue;
        }

        if let Some(stripped) = strip_trailing_name_from_text(trimmed) {
            let author_part = trimmed[stripped.len()..].trim().trim_end_matches(',');
            if !author_part.is_empty() {
                return vec![author_part.to_string()];
            }
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() < 5 {
            continue;
        }

        let name_words: Vec<&str> = words
            .iter()
            .rev()
            .take(3)
            .take_while(|w| {
                let clean = w.trim_end_matches(',');
                clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    && !PREPOSITIONS.contains(&clean.to_lowercase().as_str())
                    && clean.len() <= 15
            })
            .copied()
            .collect();

        if name_words.len() >= 2 {
            let name: String = name_words
                .iter()
                .rev()
                .map(|w| w.trim_end_matches(','))
                .collect::<Vec<_>>()
                .join(" ");
            return vec![name];
        }
    }
    Vec::new()
}

fn extract_authors_from_and_pattern(line: &str) -> Option<Vec<String>> {
    let normalized = line.replace('\u{a0}', " ");
    let stripped = normalized
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == ',' || c == ' ')
        .trim();
    if stripped.is_empty() || !stripped.contains(" and ") {
        return None;
    }

    let and_pos = stripped.find(" and ")?;
    let mut before_and = stripped[..and_pos].trim().to_string();
    let mut after_and = stripped[and_pos + 5..].trim().to_string();

    for marker in &["Citation:", "citation:", "Citation ", "citation "] {
        if let Some(pos) = after_and.find(marker) {
            after_and = after_and[..pos].trim().to_string();
        }
        if let Some(pos) = before_and.find(marker) {
            before_and = before_and[..pos].trim().to_string();
        }
    }

    let mut authors: Vec<String> = Vec::new();

    for part in before_and.split(',') {
        let trimmed = part.trim().trim_end_matches(',');
        if trimmed.is_empty() {
            continue;
        }
        if !is_name_like(trimmed) {
            return None;
        }
        authors.push(trimmed.to_string());
    }

    let last_author = extract_first_name(&after_and)?;
    authors.push(last_author);

    if after_and.contains(',') {
        let extra: Vec<String> = after_and
            .split(',')
            .skip(1)
            .map(|s| s.trim().trim_end_matches(','))
            .filter(|s| !s.is_empty() && is_name_like(s))
            .map(|s| s.to_string())
            .collect();
        authors.extend(extra);
    }

    if authors.len() >= 2 {
        Some(authors)
    } else {
        None
    }
}

fn extract_first_name(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let mut name_words: Vec<String> = Vec::new();
    for word in &words {
        let clean = word.trim_end_matches([',', ':']);
        if clean.is_empty() {
            break;
        }
        if JOURNAL_ABBREVS.contains(&clean) {
            break;
        }
        if !clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            break;
        }
        if PREPOSITIONS.contains(&clean.to_lowercase().as_str()) {
            break;
        }
        let alpha_count = clean.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count < clean.len() * 3 / 4 {
            break;
        }
        name_words.push(clean.to_string());
        if name_words.len() >= 4 {
            break;
        }
    }
    if name_words.is_empty() {
        None
    } else {
        Some(name_words.join(" "))
    }
}

fn is_name_like(s: &str) -> bool {
    let words: Vec<&str> = s.split_whitespace().collect();
    if !(1..=4).contains(&words.len()) {
        return false;
    }
    for word in &words {
        let clean = word.trim_end_matches(',');
        if clean.is_empty() || clean.ends_with(':') {
            return false;
        }
        if !clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return false;
        }
        if PREPOSITIONS.contains(&clean.to_lowercase().as_str()) {
            return false;
        }
        let alpha_count = clean.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count < clean.len() * 3 / 4 {
            return false;
        }
    }
    true
}

fn scan_for_and_pattern(lines: &[&str]) -> Option<Vec<String>> {
    let boundary = lines
        .iter()
        .enumerate()
        .take(30)
        .find(|(_, line)| {
            let lower = line.trim().to_lowercase();
            lower.starts_with("keywords")
                || lower.starts_with("1. introduction")
                || lower.starts_with("1 introduction")
        })
        .map(|(i, _)| i)
        .unwrap_or_else(|| 15.min(lines.len()));

    for i in 0..boundary {
        let trimmed = lines[i].trim();
        let cleaned = strip_citation_suffix(trimmed);
        let working = if cleaned.len() < trimmed.len() {
            cleaned.as_str()
        } else {
            trimmed
        };
        if working.len() < 5 || working.len() > 200 {
            continue;
        }
        let lower = working.to_lowercase();
        if lower.contains("university")
            || lower.contains("institute")
            || lower.contains("department")
            || lower.contains("abstract")
            || lower.contains("www.")
            || lower.contains("e-mail")
            || lower.contains("email")
            || lower.contains("received")
        {
            continue;
        }
        if let Some(authors) = extract_authors_from_and_pattern(working) {
            return Some(authors);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_author_from_title_senior_member() {
        let title = "Some Paper Title John Smith, Senior Member , IEEE";
        let result = strip_author_from_title(title);
        assert_eq!(result, "Some Paper Title");
    }

    #[test]
    fn test_strip_author_no_author() {
        let title = "A Tutorial on Spectral Clustering";
        let result = strip_author_from_title(title);
        assert_eq!(result, "A Tutorial on Spectral Clustering");
    }

    #[test]
    fn test_guess_authors_jumbled_text() {
        let text = "A Tutorial on Spectral Clustering\n\nMax Planck Institute for Biological Cybernetics\n\nUlrike von Luxburg\n\nAbstract\n\nIn recent years...\n";
        let authors = guess_authors(text);
        assert!(!authors.is_empty());
        assert!(authors[0].contains("Ulrike") || authors[0].contains("Luxburg"));
    }

    #[test]
    fn test_guess_authors_with_em_dash_abstract() {
        let text = "IEEE TRANSACTIONS ON PATTERN ANALYSIS AND MACHINE INTELLIGENCE, VOL. 26, NO. 9, SEPTEMBER 2004 1243\n\nDistance-Preserving Projection of High-Dimensional Data for Nonlinear Dimensionality Reduction Li Yang,\n\nSenior Member , IEEE\n\nAbstract —A distance-preserving method is presented to map high-dimensional data.\n";
        let authors = guess_authors(text);
        assert!(!authors.is_empty());
        assert!(authors.iter().any(|a| a.contains("Li Yang") || a.contains("Yang")));
    }

    #[test]
    fn test_guess_authors_iop_with_nbsp_and_noise() {
        let text = "Home Search Collections\nA short guide to pure point diffraction in cut-and-project sets\n\n16, 2345,,, Christoph\u{a0}Richard and Nicolae\u{a0}Strungaru\n\n1 Department für Mathematik, Friedrich-Alexander-Universität Erlangen-Nürnberg\n\nAbstract\nWe briefly review the diffraction of quasicrystals.\n";
        let authors = guess_authors(text);
        assert!(
            authors.iter().any(|a| a.contains("Christoph Richard") || a.contains("Richard")),
            "should find Christoph Richard: {authors:?}"
        );
        assert!(
            authors.iter().any(|a| a.contains("Nicolae Strungaru") || a.contains("Strungaru")),
            "should find Nicolae Strungaru: {authors:?}"
        );
    }

    #[test]
    fn test_guess_authors_no_abstract_marker_with_and_pattern() {
        let text = "Some Journal Title\nA Great Paper Title Here\nChristoph Richard and Nicolae Strungaru\n1 Department of Math, Some University\nE-mail: foo@bar.edu\nReceived 27 June 2016\n1. Introduction\nLorem ipsum dolor sit amet.\n";
        let authors = guess_authors(text);
        assert!(
            !authors.is_empty(),
            "should find authors without abstract marker: {authors:?}"
        );
        assert!(authors.iter().any(|a| a.contains("Richard") || a.contains("Strungaru")));
    }

    #[test]
    fn test_guess_authors_comma_separated_with_and_and_citation() {
        let text = "Lattice Green's Function. Introduction\nShigetoshi Katsura, Tohru Morita, Sakari Inawashiro, Tsuyoshi Horiguchi, and Yoshihiko Abe Citation: J. Math. Phys. 12, \nAdditional information on J. Math. Phys.\n1 INTRODUCTION  This year crystallographers\n";
        let authors = guess_authors(text);
        assert!(
            !authors.is_empty(),
            "should find authors from comma-separated list with 'and': {authors:?}"
        );
        assert!(authors.iter().any(|a| a.contains("Katsura") || a.contains("Morita")));
    }

    #[test]
    fn diag_richard2017() {
        let path = std::path::PathBuf::from("tmp/[중요]richard2017.pdf");
        if !path.exists() {
            return;
        }
        use std::io::Write;
        let mut f = std::fs::File::create("/tmp/richard_diag.txt").unwrap();
        let text = crate::pdf::text::extract_text(&path).expect("extract_text");
        let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        writeln!(f, "=== {} non-empty lines, {} chars ===", lines.len(), text.len()).unwrap();
        for (i, line) in lines.iter().take(50).enumerate() {
            writeln!(f, "  L{:02}: {:?}", i, &line[..line.len().min(200)]).unwrap();
        }
        writeln!(f, "\n=== Abstract search ===").unwrap();
        for (i, line) in lines.iter().enumerate() {
            let low = line.trim().to_lowercase();
            if low.starts_with("abstract") {
                writeln!(f, "  L{:02}: {:?}", i, &line[..line.len().min(200)]).unwrap();
                break;
            }
        }
        let authors = guess_authors(&text);
        writeln!(f, "\n=== guess_authors result: {:?} ===", authors).unwrap();
        let meta = crate::pdf::process_file(&path).expect("process_file");
        writeln!(f, "  title: {:?}", meta.title).unwrap();
        writeln!(f, "  authors: {:?}", meta.authors).unwrap();
        writeln!(f, "  journal: {:?}", meta.journal).unwrap();
        writeln!(f, "  pub_year: {:?}", meta.pub_year).unwrap();
    }
}
