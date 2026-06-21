//! Shared formatting helpers for citation style templates.

use crate::db::documents::split_authors;

/// Parses "Last, First", "First Last", or "Last, F." into (last_name, first_initial).
pub fn parse_author(name: &str) -> (String, String) {
    let name = name.trim();
    if name.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim().to_string();
        let first_part = name[comma_pos + 1..].trim();
        let initial = first_initial(first_part);
        (last, initial)
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
pub fn parse_author_full(name: &str) -> (String, String) {
    let name = name.trim();
    if name.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim().to_string();
        let first = name[comma_pos + 1..].trim().to_string();
        (last, first)
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
pub fn first_initial(first: &str) -> String {
    for ch in first.trim().chars() {
        if ch.is_alphabetic() {
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
    authors
        .map(|s| split_authors(s))
        .unwrap_or_default()
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
            let (last, initial) = parse_author(name);
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
pub fn format_authors_full(
    authors: &[String],
    delimiter: &str,
    last_delimiter: &str,
) -> String {
    let formatted: Vec<String> = authors
        .iter()
        .map(|name| {
            let (last, first) = parse_author_full(name);
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
            let (last, first) = parse_author_full(name);
            if first.is_empty() {
                last
            } else {
                let initials: String = first
                    .split_whitespace()
                    .filter_map(|w| {
                        let c = w.chars().next();
                        c.filter(|c| c.is_alphabetic()).map(|c| {
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
            let (last, first) = parse_author_full(name);
            if first.is_empty() {
                last
            } else {
                let initials: String = first
                    .split_whitespace()
                    .filter_map(|w| {
                        let c = w.chars().next();
                        c.filter(|c| c.is_alphabetic()).map(|c| {
                            c.to_uppercase().collect::<String>()
                        })
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
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"),
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
        let (last, initial) = parse_author("Smith, John");
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_parse_author_no_comma() {
        let (last, initial) = parse_author("John Smith");
        assert_eq!(last, "Smith");
        assert_eq!(initial, "J");
    }

    #[test]
    fn test_parse_author_full_comma() {
        let (last, first) = parse_author_full("Smith, John");
        assert_eq!(last, "Smith");
        assert_eq!(first, "John");
    }

    #[test]
    fn test_parse_author_full_no_comma() {
        let (last, first) = parse_author_full("John Adam Smith");
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
}
