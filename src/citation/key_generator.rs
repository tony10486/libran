use crate::db::documents::Document;

#[derive(Clone, Debug)]
#[derive(Default)]
pub enum CitationKeyMode {
    #[default]
    AuthorYear,
    AuthorYearTitle,
    AuthorYearHash,
    Custom(String),
}


pub fn generate_citation_key(
    doc: &Document,
    mode: &CitationKeyMode,
    exists: impl Fn(&str) -> bool,
) -> String {
    let base = match mode {
        CitationKeyMode::AuthorYear => build_author_year(doc),
        CitationKeyMode::AuthorYearTitle => build_author_year_title(doc),
        CitationKeyMode::AuthorYearHash => build_author_year_hash(doc),
        CitationKeyMode::Custom(template) => build_custom(doc, template),
    };

    let sanitized = sanitize_key(&base);
    resolve_collision(&sanitized, &exists)
}

fn build_author_year(doc: &Document) -> String {
    let author = first_author_surname(doc).unwrap_or_else(|| "Anon".to_string());
    let year = doc.pub_year.map(|y| y.to_string()).unwrap_or_else(|| "nd".to_string());
    format!("{}{}", author, year)
}

fn build_author_year_title(doc: &Document) -> String {
    let base = build_author_year(doc);
    let title_word = first_title_word(doc).unwrap_or_default();
    if title_word.is_empty() {
        base
    } else {
        format!("{}{}", base, title_word)
    }
}

fn build_author_year_hash(doc: &Document) -> String {
    let base = build_author_year(doc);
    let hash = short_hash(&doc.title);
    format!("{}{}", base, hash)
}

fn build_custom(doc: &Document, template: &str) -> String {
    let author = first_author_surname(doc).unwrap_or_default();
    let year = doc.pub_year.map(|y| y.to_string()).unwrap_or_else(|| "nd".to_string());
    let year2 = if year.len() >= 4 { &year[2..] } else { "nd" };
    let title_word = first_title_word(doc).unwrap_or_default();

    template
        .replace("{author}", &author)
        .replace("{year}", &year)
        .replace("{year2}", year2)
        .replace("{titleword}", &title_word)
        .replace("{title}", &truncate_title(&doc.title))
}

fn first_author_surname(doc: &Document) -> Option<String> {
    let authors = doc.authors.as_ref()?;
    let first = authors.split(';').next()?;
    let first = first.trim();
    if first.is_empty() {
        return None;
    }

    if first.contains(',') {
        return Some(first.split(',').next()?.trim().to_string());
    }

    let parts: Vec<&str> = first.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    Some(parts.last()?.to_string())
}

fn first_title_word(doc: &Document) -> Option<String> {
    let words: Vec<&str> = doc.title.split_whitespace().collect();
    let stop_words = ["the", "a", "an", "of", "in", "on", "for", "and", "with", "to", "at", "by"];
    for word in words {
        let lower = word.to_lowercase();
        let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());
        if !stop_words.contains(&clean) && !clean.is_empty() {
            return Some(capitalize(clean));
        }
    }
    None
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn truncate_title(title: &str) -> String {
    title.split_whitespace().take(5).collect::<Vec<_>>().join("")
}

fn short_hash(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    hex_short(&result[..3])
}

fn hex_short(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ':' || *c == '.')
        .collect()
}

fn resolve_collision(base: &str, exists: &impl Fn(&str) -> bool) -> String {
    if !exists(base) {
        return base.to_string();
    }

    let suffixes = "abcdefghijklmnopqrstuvwxyz";
    for suffix in suffixes.chars() {
        let candidate = format!("{}{}", base, suffix);
        if !exists(&candidate) {
            return candidate;
        }
    }

    for s1 in suffixes.chars() {
        for s2 in suffixes.chars() {
            let candidate = format!("{}{}{}", base, s1, s2);
            if !exists(&candidate) {
                return candidate;
            }
        }
    }

    format!("{}_{}", base, short_hash(base))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::documents::Document;

    fn make_doc(title: &str, authors: &str, year: Option<i64>) -> Document {
        Document {
            id: None,
            title: title.to_string(),
            authors: Some(authors.to_string()),
            journal: None,
            conference: None,
            pub_year: year,
            doi: None,
            arxiv_id: None,
            abstract_text: None,
            keywords: None,
            file_path: None,
            file_hash: None,
            citation_key: None,
            source: None,
            rating: None,
        }
    }

    #[test]
    fn test_author_year() {
        let doc = make_doc("Some Paper", "Smith, John", Some(2024));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |_| false);
        assert_eq!(key, "Smith2024");
    }

    #[test]
    fn test_author_year_no_year() {
        let doc = make_doc("Some Paper", "Smith, John", None);
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |_| false);
        assert_eq!(key, "Smithnd");
    }

    #[test]
    fn test_author_year_title() {
        let doc = make_doc("The Analysis of Networks", "Kim, D.", Some(2023));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYearTitle, |_| false);
        assert_eq!(key, "Kim2023Analysis");
    }

    #[test]
    fn test_collision_single_suffix() {
        let doc = make_doc("Paper", "Smith, J.", Some(2024));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |k| k == "Smith2024");
        assert_eq!(key, "Smith2024a");
    }

    #[test]
    fn test_collision_double_suffix() {
        let doc = make_doc("Paper", "Smith, J.", Some(2024));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |k| {
            k == "Smith2024" || k == "Smith2024a"
        });
        assert_eq!(key, "Smith2024b");
    }

    #[test]
    fn test_custom_template() {
        let doc = make_doc("Network Analysis", "Lee, S.", Some(2024));
        let key = generate_citation_key(
            &doc,
            &CitationKeyMode::Custom("{author}_{year2}_{titleword}".to_string()),
            |_| false,
        );
        assert_eq!(key, "Lee_24_Network");
    }

    #[test]
    fn test_stop_word_skipped() {
        let doc = make_doc("The Art of Programming", "Brown, A.", Some(2022));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYearTitle, |_| false);
        assert_eq!(key, "Brown2022Art");
    }

    #[test]
    fn test_east_asian_author() {
        let doc = make_doc("논문 제목", "김, 대영", Some(2024));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |_| false);
        assert_eq!(key, "김2024");
    }

    #[test]
    fn test_author_year_hash_unique() {
        let doc = make_doc("Unique Paper", "Smith, J.", Some(2024));
        let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYearHash, |_| false);
        assert!(key.starts_with("Smith2024"));
        assert!(key.len() > "Smith2024".len());
    }
}
