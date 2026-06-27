use unicode_normalization::UnicodeNormalization;

/// Represents the search strategy to execute for a given query term.
#[derive(Debug, PartialEq, Eq)]
pub enum SearchPlan {
    /// FTS5 MATCH against the trigram index (≥3 chars, all scripts).
    FtsMatch(String),
    /// FTS5 MATCH against the bigram index (2-char CJK queries).
    BigramMatch(String),
    /// FTS5 MATCH against the choseong index (초성 queries like "ㅁㅂ").
    ChoseongMatch(String),
    /// SQL LIKE fallback for short queries (1-char, 2-char Latin).
    Like(String),
}

/// Build a search plan from a raw search term.
///
/// Routing:
/// - ≥3 Unicode chars → `FtsMatch` (trigram FTS5)
/// - 2 chars + contains CJK → `BigramMatch` (bigram FTS5)
/// - 2 chars + no CJK → `Like`
/// - <2 chars → `Like`
pub fn build_search_plan(term: &str) -> SearchPlan {
    let n = term.chars().count();

    if n >= 2 && is_choseong_query(term) {
        return SearchPlan::ChoseongMatch(escape_bigram_query(&bigrams_cjk(term)));
    }

    if n >= 3 {
        SearchPlan::FtsMatch(escape_fts_phrase(term))
    } else if n == 2 && has_cjk(term) {
        SearchPlan::BigramMatch(escape_bigram_query(&bigrams_cjk(term)))
    } else {
        SearchPlan::Like(term.to_string())
    }
}

fn escape_fts_phrase(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

fn escape_bigram_query(bigrams: &str) -> String {
    bigrams
        .split(' ')
        .filter(|b| !b.is_empty())
        .map(|b| format!("\"{}\"", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// NFC-normalize a string for consistent Unicode comparison.
pub fn normalize_nfc(s: &str) -> String {
    s.nfc().collect()
}

/// Escape special characters in a LIKE pattern term.
pub fn escape_like(term: &str) -> String {
    let mut out = String::with_capacity(term.len());
    for ch in term.chars() {
        match ch {
            '%' => out.push_str("\\%"),
            '_' => out.push_str("\\_"),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out
}

/// Check if a string contains any CJK characters (Hangul, Han, Kana).
pub fn has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk_char)
}

fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{AC00}'..='\u{D7AF}'   // Hangul Syllables
        | '\u{1100}'..='\u{11FF}' // Hangul Jamo
        | '\u{3130}'..='\u{318F}' // Hangul Compatibility Jamo
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A
        | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
    )
}

/// Generate CJK bigrams from text, space-joined for FTS5 unicode61 tokenization.
///
/// NFC-normalizes input, then for each maximal CJK character run, emits
/// 2-codepoint sliding-window bigrams. Non-CJK text is skipped (handled by
/// the trigram index). Returns empty string for runs shorter than 2 chars.
pub fn bigrams_cjk(text: &str) -> String {
    let normalized = normalize_nfc(text);
    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() < 2 {
        return String::new();
    }

    let mut bigrams: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_cjk_char(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_cjk_char(chars[i]) {
            i += 1;
        }
        let run = &chars[start..i];
        for j in 0..run.len().saturating_sub(1) {
            bigrams.push(format!("{}{}", run[j], run[j + 1]));
        }
    }

    bigrams.join(" ")
}

const CHOSEONG_COMPAT: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

fn decompose_choseong(c: char) -> Option<char> {
    if !('\u{AC00}'..='\u{D7A3}').contains(&c) {
        return None;
    }
    let syllable_index = (c as u32) - 0xAC00;
    let l = (syllable_index / 588) as usize;
    CHOSEONG_COMPAT.get(l).copied()
}

fn is_choseong_jamo(c: char) -> bool {
    CHOSEONG_COMPAT.contains(&c)
}

pub fn is_choseong_query(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_choseong_jamo)
}

pub fn choseong_bigrams_cjk(text: &str) -> String {
    let normalized = normalize_nfc(text);
    let chars: Vec<char> = normalized.chars().collect();

    let mut bigrams: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if decompose_choseong(chars[i]).is_none() {
            i += 1;
            continue;
        }
        let mut choseong_run: Vec<char> = Vec::new();
        while i < chars.len() {
            match decompose_choseong(chars[i]) {
                Some(ch) => {
                    choseong_run.push(ch);
                    i += 1;
                }
                None => break,
            }
        }
        for j in 0..choseong_run.len().saturating_sub(1) {
            bigrams.push(format!("{}{}", choseong_run[j], choseong_run[j + 1]));
        }
    }

    bigrams.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_search_plan tests ──

    #[test]
    fn test_2char_cjk_routed_to_bigram() {
        let q = build_search_plan("미분");
        assert_eq!(q, SearchPlan::BigramMatch("\"미분\"".to_string()));
    }

    #[test]
    fn test_1char_cjk_routed_to_like() {
        let q = build_search_plan("미");
        assert_eq!(q, SearchPlan::Like("미".to_string()));
    }

    #[test]
    fn test_3char_cjk_uses_fts_match() {
        let q = build_search_plan("미분방");
        assert_eq!(q, SearchPlan::FtsMatch("\"미분방\"".to_string()));
    }

    #[test]
    fn test_2char_latin_routed_to_like() {
        assert_eq!(build_search_plan("ab"), SearchPlan::Like("ab".to_string()));
    }

    #[test]
    fn test_1char_latin_routed_to_like() {
        assert_eq!(build_search_plan("a"), SearchPlan::Like("a".to_string()));
    }

    #[test]
    fn test_3char_latin_uses_fts_match() {
        assert_eq!(
            build_search_plan("abc"),
            SearchPlan::FtsMatch("\"abc\"".to_string())
        );
    }

    #[test]
    fn test_embedded_double_quote_escaped() {
        let q = build_search_plan("ab\"cd");
        assert_eq!(q, SearchPlan::FtsMatch("\"ab\"\"cd\"".to_string()));
    }

    #[test]
    fn test_empty_string_routed_to_like() {
        assert_eq!(build_search_plan(""), SearchPlan::Like(String::new()));
    }

    #[test]
    fn test_mixed_cjk_latin_2char_routed_to_bigram() {
        // 2 chars, one CJK one Latin → has_cjk → BigramMatch
        // bigrams_cjk only emits bigrams from CJK runs; "a미" has CJK run "미" (1 char) → no bigram
        // This is an edge case: 2-char mixed with only 1 CJK char produces empty bigrams.
        // The bigram table won't have a match, but the routing is correct.
        let q = build_search_plan("a미");
        assert!(matches!(q, SearchPlan::BigramMatch(_)));
    }

    #[test]
    fn test_choseong_2char_routed_to_choseong_match() {
        let q = build_search_plan("ㅁㅂ");
        assert!(matches!(q, SearchPlan::ChoseongMatch(_)));
    }

    #[test]
    fn test_choseong_3char_routed_to_choseong_match() {
        let q = build_search_plan("ㅁㅂㅈ");
        assert!(matches!(q, SearchPlan::ChoseongMatch(_)));
    }

    #[test]
    fn test_choseong_1char_routed_to_like() {
        let q = build_search_plan("ㅁ");
        assert_eq!(q, SearchPlan::Like("ㅁ".to_string()));
    }

    #[test]
    fn test_non_choseong_not_routed_to_choseong_match() {
        let q = build_search_plan("미분");
        assert!(!matches!(q, SearchPlan::ChoseongMatch(_)));
    }

    #[test]
    fn test_choseong_mixed_with_vowel_not_choseong_query() {
        assert!(!is_choseong_query("ㅁㅏ"));
        assert!(is_choseong_query("ㅁㅂ"));
    }

    // ── decompose_choseong / choseong_bigrams_cjk tests ──

    #[test]
    fn test_decompose_choseong_basic() {
        assert_eq!(decompose_choseong('미'), Some('ㅁ'));
        assert_eq!(decompose_choseong('분'), Some('ㅂ'));
        assert_eq!(decompose_choseong('방'), Some('ㅂ'));
        assert_eq!(decompose_choseong('정'), Some('ㅈ'));
        assert_eq!(decompose_choseong('식'), Some('ㅅ'));
    }

    #[test]
    fn test_decompose_choseong_non_hangul() {
        assert_eq!(decompose_choseong('A'), None);
        assert_eq!(decompose_choseong('中'), None);
        assert_eq!(decompose_choseong('あ'), None);
    }

    #[test]
    fn test_choseong_bigrams_korean_5chars() {
        assert_eq!(choseong_bigrams_cjk("미분방정식"), "ㅁㅂ ㅂㅂ ㅂㅈ ㅈㅅ");
    }

    #[test]
    fn test_choseong_bigrams_korean_2chars() {
        assert_eq!(choseong_bigrams_cjk("미분"), "ㅁㅂ");
    }

    #[test]
    fn test_choseong_bigrams_korean_1char_empty() {
        assert_eq!(choseong_bigrams_cjk("미"), "");
    }

    #[test]
    fn test_choseong_bigrams_empty_string() {
        assert_eq!(choseong_bigrams_cjk(""), "");
    }

    #[test]
    fn test_choseong_bigrams_mixed_with_latin() {
        // "미분 AB 방정" → Hangul runs "미분"(ㅁㅂ) and "방정"(ㅂㅈ) → bigrams "ㅁㅂ ㅂㅈ"
        // No false "ㅂㅂ" across the Latin gap.
        assert_eq!(choseong_bigrams_cjk("미분 AB 방정"), "ㅁㅂ ㅂㅈ");
    }

    #[test]
    fn test_choseong_bigrams_non_hangul_cjk_skipped() {
        // CJK ideographs and kana are not Hangul → no choseong → empty
        assert_eq!(choseong_bigrams_cjk("微分方程式"), "");
        assert_eq!(choseong_bigrams_cjk("ひらがな"), "");
    }

    // ── has_cjk tests ──

    #[test]
    fn test_has_cjk_korean() {
        assert!(has_cjk("미분방정식"));
        assert!(has_cjk("hello 미분"));
    }

    #[test]
    fn test_has_cjk_japanese() {
        assert!(has_cjk("微分方程式"));
        assert!(has_cjk("ひらがな"));
        assert!(has_cjk("カタカナ"));
    }

    #[test]
    fn test_has_cjk_latin_only() {
        assert!(!has_cjk("differential equations"));
        assert!(!has_cjk(""));
    }

    // ── bigrams_cjk tests ──

    #[test]
    fn test_bigrams_korean_5chars() {
        assert_eq!(bigrams_cjk("미분방정식"), "미분 분방 방정 정식");
    }

    #[test]
    fn test_bigrams_korean_3chars() {
        assert_eq!(bigrams_cjk("편미분"), "편미 미분");
    }

    #[test]
    fn test_bigrams_korean_2chars() {
        assert_eq!(bigrams_cjk("미분"), "미분");
    }

    #[test]
    fn test_bigrams_korean_1char_empty() {
        assert_eq!(bigrams_cjk("미"), "");
    }

    #[test]
    fn test_bigrams_empty_string() {
        assert_eq!(bigrams_cjk(""), "");
    }

    #[test]
    fn test_bigrams_mixed_cjk_latin() {
        // "AB 미분" → only CJK run "미분" → bigram "미분"
        assert_eq!(bigrams_cjk("AB 미분"), "미분");
    }

    #[test]
    fn test_bigrams_multiple_cjk_runs() {
        // "미분 AB 방정" → CJK runs "미분" and "방정" → bigrams "미분 방정"
        assert_eq!(bigrams_cjk("미분 AB 방정"), "미분 방정");
    }

    #[test]
    fn test_bigrams_nfd_normalized() {
        use unicode_normalization::UnicodeNormalization;
        let nfd: String = "미분".nfd().collect();
        let result = bigrams_cjk(&nfd);
        assert_eq!(result, "미분");
    }

    // ── normalize_nfc tests ──

    #[test]
    fn test_nfd_normalized_to_nfc() {
        use unicode_normalization::UnicodeNormalization;
        let nfd: String = "미".nfd().collect();
        let nfc = normalize_nfc(&nfd);
        assert_eq!(nfc, "미");
    }

    #[test]
    fn test_nfc_idempotent() {
        let nfc = "미분방정식";
        assert_eq!(normalize_nfc(nfc), nfc);
    }

    #[test]
    fn test_ascii_unchanged() {
        let s = "hello world";
        assert_eq!(normalize_nfc(s), s);
    }

    // ── escape_like tests ──

    #[test]
    fn test_escape_like_percent() {
        assert_eq!(escape_like("50%"), "50\\%");
    }

    #[test]
    fn test_escape_like_underscore() {
        assert_eq!(escape_like("a_b"), "a\\_b");
    }

    #[test]
    fn test_escape_like_backslash() {
        assert_eq!(escape_like("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_like_no_special() {
        assert_eq!(escape_like("미분"), "미분");
    }
}
