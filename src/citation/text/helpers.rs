//! Shared formatting helpers for citation style templates.

use crate::db::documents::split_authors;

/// Detects whether the name contains CJK characters (Korean Hangul,
/// Japanese Kana, or CJK Ideographs).
/// Used to suppress initials and avoid ambiguous whitespace splitting
/// for names that do not follow the Western "First Last" convention.
#[cfg(test)]
fn detect_cjk(name: &str) -> bool {
    name.chars().any(is_cjk_char)
}

/// Returns true if the character falls within CJK Unicode ranges.
pub fn is_cjk_char(ch: char) -> bool {
    let c = ch as u32;
    matches!(
        c,
        0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0x3400..=0x4DBF // CJK Extension A
        | 0xAC00..=0xD7AF // Hangul Syllables
        | 0x3040..=0x309F // Hiragana
        | 0x30A0..=0x30FF // Katakana
    )
}

/// Detects the CJK locale of a name from its Unicode character ranges.
/// Returns "ko" for Hangul, "ja" for Kana, "zh" for Han-only.
/// Hangul and Kana take priority over Han because they are unambiguous.
/// This replaces the coarser `detect_cjk` bool check with per-creator
/// locale detection (ko/ja/zh), enabling locale-specific rendering.
pub fn detect_locale(name: &str) -> Option<&'static str> {
    let mut has_hangul = false;
    let mut has_kana = false;
    let mut has_han = false;
    for ch in name.chars() {
        let c = ch as u32;
        if (0xAC00..=0xD7AF).contains(&c) {
            has_hangul = true;
        } else if (0x3040..=0x309F).contains(&c) || (0x30A0..=0x30FF).contains(&c) {
            has_kana = true;
        } else if (0x4E00..=0x9FFF).contains(&c) || (0x3400..=0x4DBF).contains(&c) {
            has_han = true;
        }
    }
    if has_hangul {
        Some("ko")
    } else if has_kana {
        Some("ja")
    } else if has_han {
        Some("zh")
    } else {
        None
    }
}

/// Parses "Last, First", "First Last", or "Last, F." into (last_name, first_initial).
/// For CJK names (locale is Some("ko"/"ja"/"zh") or auto-detected), the whole name
/// is treated as the family name when there is no comma. For CJK names with a
/// comma, the split is preserved but initials are suppressed.
/// When `locale` is None, falls back to `detect_locale` for per-creator detection.
pub fn parse_author(name: &str, locale: Option<&str>) -> (String, String) {
    let name = name.trim();
    if name.is_empty() {
        return (String::new(), String::new());
    }

    let is_cjk = locale.or_else(|| detect_locale(name)).is_some();

    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim().to_string();
        let first_part = name[comma_pos + 1..].trim();
        let initial = if is_cjk {
            String::new()
        } else {
            first_initial(first_part)
        };
        (last, initial)
    } else if is_cjk {
        (name.to_string(), String::new())
    } else {
        let words: Vec<&str> = name.split_whitespace().collect();
        match words.len() {
            0 => (String::new(), String::new()),
            1 => (words[0].to_string(), String::new()),
            _ => {
                let last = words.last().unwrap().to_string();
                let first_part = words[..words.len() - 1].join(" ");
                let initial = first_initial(&first_part);
                (last, initial)
            }
        }
    }
}

/// Parses "Last, First" or "First Last" into (last_name, full_first_name).
/// For styles that use full first names (APSA, ASA, Chicago, MLA).
/// For CJK names (locale is Some or auto-detected) without a comma, the whole
/// name is treated as the family name. CJK names with a comma preserve the
/// family/given split. When `locale` is None, falls back to `detect_locale`.
pub fn parse_author_full(name: &str, locale: Option<&str>) -> (String, String) {
    let name = name.trim();
    if name.is_empty() {
        return (String::new(), String::new());
    }

    let is_cjk = locale.or_else(|| detect_locale(name)).is_some();

    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim().to_string();
        let first = name[comma_pos + 1..].trim().to_string();
        (last, first)
    } else if is_cjk {
        (name.to_string(), String::new())
    } else {
        let words: Vec<&str> = name.split_whitespace().collect();
        match words.len() {
            0 => (String::new(), String::new()),
            1 => (words[0].to_string(), String::new()),
            _ => {
                let last = words.last().unwrap().to_string();
                let first = words[..words.len() - 1].join(" ");
                (last, first)
            }
        }
    }
}

/// Extracts the first alphabetic character as an uppercase initial.
/// Returns empty string for CJK characters (initials are not used in
/// Korean/Chinese/Japanese naming conventions).
pub fn first_initial(first: &str) -> String {
    for ch in first.trim().chars() {
        if ch.is_alphabetic() && !is_cjk_char(ch) {
            return ch.to_uppercase().collect();
        }
    }
    String::new()
}

/// Formats a page range from optional start/end values.
/// Returns "start-end", "start", or empty string.
pub fn format_pages(start: Option<&str>, end: Option<&str>) -> String {
    match (start, end) {
        (Some(s), Some(e)) => {
            if s == e {
                s.to_string()
            } else {
                format!("{}-{}", s, e)
            }
        }
        (Some(s), None) => s.to_string(),
        (None, Some(e)) => e.to_string(),
        (None, None) => String::new(),
    }
}

/// Formats a year, returning "2023" or "n.d." for no date.
pub fn format_year(year: Option<i64>) -> String {
    match year {
        Some(y) => y.to_string(),
        None => "n.d.".to_string(),
    }
}

/// Joins items with a delimiter, using a special last delimiter before the final item.
/// Example: join_with_delimiter(&["a", "b", "c"], ", ", " and ") → "a, b and c"
pub fn join_with_delimiter(items: &[&str], delimiter: &str, last_delimiter: &str) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        2 => format!("{}{}{}", items[0], last_delimiter, items[1]),
        _ => {
            let (last, rest) = items.split_last().unwrap();
            format!("{}{}{}", rest.join(delimiter), last_delimiter, last)
        }
    }
}

/// Splits the authors field into individual author name strings.
/// Returns empty vec if authors is None or empty.
pub fn get_authors(authors: Option<&str>) -> Vec<String> {
    authors.map(split_authors).unwrap_or_default()
}

/// Formats author names as "Last, F. M." (initials with periods) for a given list.
/// Each author becomes "Last, F." or "Last, F. M." depending on available names.
/// Joined with `delimiter`, using `last_delimiter` before the final author.
pub fn format_authors_initials(
    authors: &[String],
    delimiter: &str,
    last_delimiter: &str,
) -> String {
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, initial) = parse_author(name, detect_locale(name));
            if initial.is_empty() {
                last
            } else {
                format!("{}, {}.", last, initial)
            }
        })
        .collect();
    let refs: Vec<&str> = formatted.iter().map(|s| s.as_str()).collect();
    join_with_delimiter(&refs, delimiter, last_delimiter)
}

/// Formats author names as "Last, Full First" (full first names) for a given list.
/// Joined with `delimiter`, using `last_delimiter` before the final author.
pub fn format_authors_full(authors: &[String], delimiter: &str, last_delimiter: &str) -> String {
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name, detect_locale(name));
            if first.is_empty() {
                last
            } else {
                format!("{}, {}", last, first)
            }
        })
        .collect();
    let refs: Vec<&str> = formatted.iter().map(|s| s.as_str()).collect();
    join_with_delimiter(&refs, delimiter, last_delimiter)
}

/// Formats author names as "F. M. Last" (initials first) for a given list.
/// Used by IEEE style.
pub fn format_authors_initials_first(
    authors: &[String],
    delimiter: &str,
    last_delimiter: &str,
) -> String {
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name, detect_locale(name));
            if first.is_empty() {
                last
            } else {
                let initials: String = first
                    .split_whitespace()
                    .filter_map(|w| {
                        let c = w.chars().next();
                        c.filter(|c| c.is_alphabetic() && !is_cjk_char(*c))
                            .map(|c| {
                                let up: String = c.to_uppercase().collect();
                                format!("{}.", up)
                            })
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                if initials.is_empty() {
                    last
                } else {
                    format!("{} {}", initials, last)
                }
            }
        })
        .collect();
    let refs: Vec<&str> = formatted.iter().map(|s| s.as_str()).collect();
    join_with_delimiter(&refs, delimiter, last_delimiter)
}

/// Formats author names as "Last FM" (no periods after initials, no spaces in initials).
/// Used by AMA and Vancouver styles.
pub fn format_authors_no_period_initials(
    authors: &[String],
    delimiter: &str,
    last_delimiter: &str,
) -> String {
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name, detect_locale(name));
            if first.is_empty() {
                last
            } else {
                let initials: String = first
                    .split_whitespace()
                    .filter_map(|w| {
                        let c = w.chars().next();
                        c.filter(|c| c.is_alphabetic() && !is_cjk_char(*c))
                            .map(|c| c.to_uppercase().collect::<String>())
                    })
                    .collect::<Vec<_>>()
                    .join("");
                if initials.is_empty() {
                    last
                } else {
                    format!("{} {}", last, initials)
                }
            }
        })
        .collect();
    let refs: Vec<&str> = formatted.iter().map(|s| s.as_str()).collect();
    join_with_delimiter(&refs, delimiter, last_delimiter)
}

/// Converts a number to Roman numerals (uppercase).
/// Used by MHRA for volume numbers.
pub fn to_roman(num: i64) -> String {
    if num <= 0 {
        return num.to_string();
    }
    let pairs = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    let mut remaining = num;
    for (value, symbol) in &pairs {
        while remaining >= *value {
            result.push_str(symbol);
            remaining -= *value;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_author_comma() {
        let (last, initial) = parse_author("Smith, John", None);
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_parse_author_no_comma() {
        let (last, initial) = parse_author("John Smith", None);
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_parse_author_full_comma() {
        let (last, first) = parse_author_full("Smith, John", None);
        assert_eq!(last, "Smith");
        assert_eq!(first, "John");
    }

    #[test]
    fn test_parse_author_full_no_comma() {
        let (last, first) = parse_author_full("John Adam Smith", None);
        assert_eq!(last, "Smith");
        assert_eq!(first, "John Adam");
    }

    #[test]
    fn test_format_pages() {
        assert_eq!(format_pages(Some("123"), Some("145")), "123-145");
        assert_eq!(format_pages(Some("123"), None), "123");
        assert_eq!(format_pages(None, None), "");
        assert_eq!(format_pages(Some("5"), Some("5")), "5");
    }

    #[test]
    fn test_format_year() {
        assert_eq!(format_year(Some(2023)), "2023");
        assert_eq!(format_year(None), "n.d.");
    }

    #[test]
    fn test_join_with_delimiter() {
        assert_eq!(join_with_delimiter(&["a"], ", ", " and "), "a");
        assert_eq!(join_with_delimiter(&["a", "b"], ", ", " and "), "a and b");
        assert_eq!(
            join_with_delimiter(&["a", "b", "c"], ", ", " and "),
            "a, b and c"
        );
    }

    #[test]
    fn test_format_authors_initials() {
        let authors = vec!["Smith, John".to_string(), "Lee, Jane".to_string()];
        let result = format_authors_initials(&authors, ", ", ", & ");
        assert_eq!(result, "Smith, J., & Lee, J.");
    }

    #[test]
    fn test_format_authors_full() {
        let authors = vec!["Smith, John A.".to_string(), "Jones, Bob C.".to_string()];
        let result = format_authors_full(&authors, ", ", " and ");
        assert_eq!(result, "Smith, John A. and Jones, Bob C.");
    }

    #[test]
    fn test_format_authors_initials_first() {
        let authors = vec!["Smith, John A.".to_string(), "Jones, Bob C.".to_string()];
        let result = format_authors_initials_first(&authors, ", ", " and ");
        assert_eq!(result, "J. A. Smith and B. C. Jones");
    }

    #[test]
    fn test_format_authors_no_period_initials() {
        let authors = vec!["Smith, John A.".to_string(), "Jones, Bob C.".to_string()];
        let result = format_authors_no_period_initials(&authors, ", ", ", ");
        assert_eq!(result, "Smith JA, Jones BC");
    }

    #[test]
    fn test_to_roman() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(42), "XLII");
        assert_eq!(to_roman(2023), "MMXXIII");
        assert_eq!(to_roman(0), "0");
        assert_eq!(to_roman(-1), "-1");
    }

    // --- CJK heuristic tests (T8 detect_cjk) ---

    #[test]
    fn test_detect_cjk_korean() {
        assert!(detect_cjk("김철수"));
        assert!(detect_cjk("김, 철수"));
    }

    #[test]
    fn test_detect_cjk_chinese() {
        assert!(detect_cjk("张三"));
        assert!(detect_cjk("李, 四"));
    }

    #[test]
    fn test_detect_cjk_japanese() {
        assert!(detect_cjk("山田太郎"));
        assert!(detect_cjk("田中, 太郎"));
        assert!(detect_cjk("やまだ")); // Hiragana
        assert!(detect_cjk("ヤマダ")); // Katakana
    }

    #[test]
    fn test_detect_cjk_western_false() {
        assert!(!detect_cjk("Smith, John"));
        assert!(!detect_cjk("John Smith"));
        assert!(!detect_cjk(""));
    }

    #[test]
    fn test_cjk_no_comma_literal() {
        // Given: a Korean name with no comma
        // When: parse_author splits it
        // Then: whole name is family, no initial (CJK is ambiguous when no comma)
        let (last, initial) = parse_author("김철수", None);
        assert_eq!(last, "김철수");
        assert_eq!(initial, "");
    }

    #[test]
    fn test_cjk_with_comma() {
        // Given: a Korean name with comma "김, 철수"
        // When: parse_author splits it
        // Then: family="김", initial="" (initials suppressed for CJK)
        let (last, initial) = parse_author("김, 철수", None);
        assert_eq!(last, "김");
        assert_eq!(initial, "");
    }

    #[test]
    fn test_cjk_with_comma_chinese() {
        let (last, initial) = parse_author("张, 三", None);
        assert_eq!(last, "张");
        assert_eq!(initial, "");
    }

    #[test]
    fn test_cjk_with_comma_japanese() {
        let (last, initial) = parse_author("田中, 太郎", None);
        assert_eq!(last, "田中");
        assert_eq!(initial, "");
    }

    #[test]
    fn test_western_unchanged_comma() {
        // Given: a Western name with comma
        // When: parse_author splits it
        // Then: normal split, initial extracted (no regression)
        let (last, initial) = parse_author("Smith, John", None);
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_western_unchanged_no_comma() {
        // Given: a Western name with no comma
        // When: parse_author splits it
        // Then: last word is family, initial extracted (no regression)
        let (last, initial) = parse_author("John Smith", None);
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_mixed_authors_cjk_and_western() {
        // Given: a mix of CJK and Western authors
        let authors = vec!["김철수".to_string(), "Smith, John".to_string()];
        // When: formatted with initials style
        let result = format_authors_initials(&authors, ", ", ", & ");
        // Then: CJK author renders as family only, Western author renders normally
        assert_eq!(result, "김철수, & Smith, J.");
    }

    #[test]
    fn test_cjk_first_initial_suppressed() {
        // Given: a CJK given name
        // When: first_initial extracts the initial
        // Then: empty string (CJK initials are suppressed)
        assert_eq!(first_initial("철수"), "");
        assert_eq!(first_initial("三"), "");
        assert_eq!(first_initial("太郎"), "");
    }

    #[test]
    fn test_western_first_initial_unchanged() {
        assert_eq!(first_initial("John"), "J");
        assert_eq!(first_initial("Adam"), "A");
    }

    // --- Per-creator locale tests (T16 detect_locale) ---

    #[test]
    fn test_detect_locale_korean() {
        assert_eq!(detect_locale("김철수"), Some("ko"));
        assert_eq!(detect_locale("김, 철수"), Some("ko"));
    }

    #[test]
    fn test_detect_locale_japanese() {
        // Kanji-only names are ambiguous (could be Chinese or Japanese) → "zh"
        assert_eq!(detect_locale("山田太郎"), Some("zh"));
        assert_eq!(detect_locale("田中, 太郎"), Some("zh"));
        // Kana-containing names are unambiguously Japanese
        assert_eq!(detect_locale("やまだ"), Some("ja"));
        assert_eq!(detect_locale("ヤマダ"), Some("ja"));
        assert_eq!(detect_locale("山田やまだ"), Some("ja"));
    }

    #[test]
    fn test_detect_locale_chinese() {
        assert_eq!(detect_locale("张三"), Some("zh"));
        assert_eq!(detect_locale("李, 四"), Some("zh"));
    }

    #[test]
    fn test_detect_locale_western_none() {
        assert_eq!(detect_locale("Smith, John"), None);
        assert_eq!(detect_locale("John Smith"), None);
        assert_eq!(detect_locale(""), None);
    }

    #[test]
    fn test_cjk_creator_korean() {
        // Given: a Korean creator with explicit locale='ko'
        // When: parse_author with Some("ko")
        // Then: family-first order, no initials
        let (last, initial) = parse_author("김철수", Some("ko"));
        assert_eq!(last, "김철수");
        assert_eq!(initial, "");

        let (last2, initial2) = parse_author("김, 철수", Some("ko"));
        assert_eq!(last2, "김");
        assert_eq!(initial2, "");
    }

    #[test]
    fn test_cjk_creator_literal() {
        // Given: a Korean creator with literal form (no comma, whole name)
        // When: parse_author_full with Some("ko")
        // Then: whole name used as-is (family), no given name
        let (last, first) = parse_author_full("김철수", Some("ko"));
        assert_eq!(last, "김철수");
        assert_eq!(first, "");

        let (last2, first2) = parse_author_full("김철수", None);
        assert_eq!(last2, "김철수");
        assert_eq!(first2, "");
    }

    #[test]
    fn test_mixed_cjk_western() {
        // Given: a mix of CJK and Western authors
        let authors = vec!["김철수".to_string(), "Smith, John".to_string()];
        // When: formatted with initials style (auto-detects locale per name)
        let result = format_authors_initials(&authors, ", ", ", & ");
        // Then: CJK author renders as family only, Western author renders normally
        assert_eq!(result, "김철수, & Smith, J.");
    }

    #[test]
    fn test_cjk_creator_japanese_full() {
        // Given: a Japanese creator with comma
        // When: parse_author_full with Some("ja")
        // Then: family/given split preserved (for full-name styles)
        let (last, first) = parse_author_full("田中, 太郎", Some("ja"));
        assert_eq!(last, "田中");
        assert_eq!(first, "太郎");
    }

    #[test]
    fn test_cjk_creator_chinese_no_comma() {
        // Given: a Chinese creator without comma
        // When: parse_author with Some("zh")
        // Then: whole name as family, no initials
        let (last, initial) = parse_author("张三", Some("zh"));
        assert_eq!(last, "张三");
        assert_eq!(initial, "");
    }

    #[test]
    fn test_cjk_initials_first_suppressed() {
        // Given: a CJK author with comma (family, given)
        // When: format_authors_initials_first processes it
        // Then: no initials extracted from CJK given name
        let authors = vec!["김, 철수".to_string()];
        let result = format_authors_initials_first(&authors, ", ", " and ");
        assert_eq!(result, "김");
    }

    #[test]
    fn test_cjk_no_period_initials_suppressed() {
        // Given: a CJK author with comma (family, given)
        // When: format_authors_no_period_initials processes it
        // Then: no initials extracted from CJK given name
        let authors = vec!["김, 철수".to_string()];
        let result = format_authors_no_period_initials(&authors, ", ", ", ");
        assert_eq!(result, "김");
    }
}
