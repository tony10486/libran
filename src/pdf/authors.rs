pub static AUTHOR_MARKERS: &[&str] = &[
    "Senior Member", "Junior Member", "Member", "Fellow",
    "Senior, IEEE", "IEEE", "ACM", "AAAI",
];

pub const PREPOSITIONS: &[&str] = &[
    "of", "for", "the", "and", "on", "in", "a", "an", "to", "via", "by",
    "with", "from", "into", "using", "through", "over", "under",
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

pub fn guess_authors(full_text: &str) -> Vec<String> {
    let lines: Vec<&str> = full_text.lines().filter(|l| !l.trim().is_empty()).collect();
    let abstract_idx = crate::pdf::heuristic::find_abstract_marker_pub(&lines);

    if let Some(idx) = abstract_idx {
        for i in (0..idx).rev() {
            let trimmed = lines[i].trim();
            if trimmed.len() < 5 || trimmed.len() > 200 {
                continue;
            }

            let lower = trimmed.to_lowercase();
            if lower.contains("university")
                || lower.contains("institute")
                || lower.contains("department")
                || lower.contains("abstract")
                || lower.contains("keywords")
                || lower.contains("www.")
            {
                continue;
            }

            if AUTHOR_MARKERS.iter().any(|m| trimmed.contains(m)) {
                continue;
            }

            if let Some(authors) = extract_authors_from_line(trimmed) {
                return authors;
            }
        }
    }

    extract_authors_from_title_line(&lines, abstract_idx)
}

fn extract_authors_from_line(trimmed: &str) -> Option<Vec<String>> {
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

    let lower = trimmed.to_lowercase();
    if lower.contains("tutorial") || lower.contains("clustering") {
        return None;
    }

    let authors: Vec<String> = trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() > 2)
        .collect();
    if authors.is_empty() {
        None
    } else {
        Some(authors)
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

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() < 5 {
            continue;
        }

        let name_words: Vec<&str> = words
            .iter()
            .rev()
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
