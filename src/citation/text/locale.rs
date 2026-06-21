//! CSL term localization for 4 languages.
//!
//! Provides localized strings for citation terms like "et al.", "and",
//! "accessed", "edition", etc. used by the style templates.

use crate::citation::text::styles::CitationLanguage;

/// Citation terms that have language-specific renderings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    EtAl,
    And,
    Accessed,
    In,
    Edition,
    EditionShort,
    Volume,
    VolumeShort,
    Issue,
    IssueShort,
    Page,
    PageShort,
    NoDate,
    NoDateShort,
    Editor,
    EditorShort,
    Translator,
    TranslatorShort,
    Ibid,
    Forthcoming,
    Anonymous,
    OpenQuote,
    CloseQuote,
    OpenInnerQuote,
    CloseInnerQuote,
}

/// Returns the localized string for `term` in `lang`.
pub fn term(term: Term, lang: CitationLanguage) -> &'static str {
    match (term, lang) {
        // English
        (Term::EtAl, CitationLanguage::English) => "et al.",
        (Term::And, CitationLanguage::English) => "and",
        (Term::Accessed, CitationLanguage::English) => "accessed",
        (Term::In, CitationLanguage::English) => "In",
        (Term::Edition, CitationLanguage::English) => "edition",
        (Term::EditionShort, CitationLanguage::English) => "ed.",
        (Term::Volume, CitationLanguage::English) => "volume",
        (Term::VolumeShort, CitationLanguage::English) => "vol.",
        (Term::Issue, CitationLanguage::English) => "number",
        (Term::IssueShort, CitationLanguage::English) => "no.",
        (Term::Page, CitationLanguage::English) => "page",
        (Term::PageShort, CitationLanguage::English) => "p.",
        (Term::NoDate, CitationLanguage::English) => "no date",
        (Term::NoDateShort, CitationLanguage::English) => "n.d.",
        (Term::Editor, CitationLanguage::English) => "editor",
        (Term::EditorShort, CitationLanguage::English) => "ed.",
        (Term::Translator, CitationLanguage::English) => "translator",
        (Term::TranslatorShort, CitationLanguage::English) => "trans.",
        (Term::Ibid, CitationLanguage::English) => "ibid.",
        (Term::Forthcoming, CitationLanguage::English) => "forthcoming",
        (Term::Anonymous, CitationLanguage::English) => "Anonymous",
        (Term::OpenQuote, CitationLanguage::English) => "\u{201C}",
        (Term::CloseQuote, CitationLanguage::English) => "\u{201D}",
        (Term::OpenInnerQuote, CitationLanguage::English) => "\u{2018}",
        (Term::CloseInnerQuote, CitationLanguage::English) => "\u{2019}",

        // Korean
        (Term::EtAl, CitationLanguage::Korean) => "외",
        (Term::And, CitationLanguage::Korean) => "와/과",
        (Term::Accessed, CitationLanguage::Korean) => "접근일",
        (Term::In, CitationLanguage::Korean) => "In",
        (Term::Edition, CitationLanguage::Korean) => "판",
        (Term::EditionShort, CitationLanguage::Korean) => "판",
        (Term::Volume, CitationLanguage::Korean) => "권",
        (Term::VolumeShort, CitationLanguage::Korean) => "vol.",
        (Term::Issue, CitationLanguage::Korean) => "호",
        (Term::IssueShort, CitationLanguage::Korean) => "no.",
        (Term::Page, CitationLanguage::Korean) => "면",
        (Term::PageShort, CitationLanguage::Korean) => "p.",
        (Term::NoDate, CitationLanguage::Korean) => "일자 없음",
        (Term::NoDateShort, CitationLanguage::Korean) => "n.d.",
        (Term::Editor, CitationLanguage::Korean) => "편집자",
        (Term::EditorShort, CitationLanguage::Korean) => "ed.",
        (Term::Translator, CitationLanguage::Korean) => "번역자",
        (Term::TranslatorShort, CitationLanguage::Korean) => "trans.",
        (Term::Ibid, CitationLanguage::Korean) => "ibid.",
        (Term::Forthcoming, CitationLanguage::Korean) => "forthcoming",
        (Term::Anonymous, CitationLanguage::Korean) => "익명",
        (Term::OpenQuote, CitationLanguage::Korean) => "\u{201C}",
        (Term::CloseQuote, CitationLanguage::Korean) => "\u{201D}",
        (Term::OpenInnerQuote, CitationLanguage::Korean) => "\u{2018}",
        (Term::CloseInnerQuote, CitationLanguage::Korean) => "\u{2019}",

        // Japanese
        (Term::EtAl, CitationLanguage::Japanese) => "ほか",
        (Term::And, CitationLanguage::Japanese) => "and",
        (Term::Accessed, CitationLanguage::Japanese) => "参照",
        (Term::In, CitationLanguage::Japanese) => "In",
        (Term::Edition, CitationLanguage::Japanese) => "版",
        (Term::EditionShort, CitationLanguage::Japanese) => "版",
        (Term::Volume, CitationLanguage::Japanese) => "巻",
        (Term::VolumeShort, CitationLanguage::Japanese) => "vol.",
        (Term::Issue, CitationLanguage::Japanese) => "号",
        (Term::IssueShort, CitationLanguage::Japanese) => "no.",
        (Term::Page, CitationLanguage::Japanese) => "ページ",
        (Term::PageShort, CitationLanguage::Japanese) => "p.",
        (Term::NoDate, CitationLanguage::Japanese) => "日付なし",
        (Term::NoDateShort, CitationLanguage::Japanese) => "n.d.",
        (Term::Editor, CitationLanguage::Japanese) => "編集者",
        (Term::EditorShort, CitationLanguage::Japanese) => "ed.",
        (Term::Translator, CitationLanguage::Japanese) => "翻訳者",
        (Term::TranslatorShort, CitationLanguage::Japanese) => "trans.",
        (Term::Ibid, CitationLanguage::Japanese) => "ibid.",
        (Term::Forthcoming, CitationLanguage::Japanese) => "forthcoming",
        (Term::Anonymous, CitationLanguage::Japanese) => "匿名",
        (Term::OpenQuote, CitationLanguage::Japanese) => "\u{300C}",
        (Term::CloseQuote, CitationLanguage::Japanese) => "\u{300D}",
        (Term::OpenInnerQuote, CitationLanguage::Japanese) => "\u{300E}",
        (Term::CloseInnerQuote, CitationLanguage::Japanese) => "\u{300F}",

        // Chinese
        (Term::EtAl, CitationLanguage::Chinese) => "等",
        (Term::And, CitationLanguage::Chinese) => "和",
        (Term::Accessed, CitationLanguage::Chinese) => "见于",
        (Term::In, CitationLanguage::Chinese) => "In",
        (Term::Edition, CitationLanguage::Chinese) => "版",
        (Term::EditionShort, CitationLanguage::Chinese) => "版",
        (Term::Volume, CitationLanguage::Chinese) => "卷",
        (Term::VolumeShort, CitationLanguage::Chinese) => "vol.",
        (Term::Issue, CitationLanguage::Chinese) => "期",
        (Term::IssueShort, CitationLanguage::Chinese) => "no.",
        (Term::Page, CitationLanguage::Chinese) => "页",
        (Term::PageShort, CitationLanguage::Chinese) => "p.",
        (Term::NoDate, CitationLanguage::Chinese) => "不详",
        (Term::NoDateShort, CitationLanguage::Chinese) => "n.d.",
        (Term::Editor, CitationLanguage::Chinese) => "编辑",
        (Term::EditorShort, CitationLanguage::Chinese) => "ed.",
        (Term::Translator, CitationLanguage::Chinese) => "翻译",
        (Term::TranslatorShort, CitationLanguage::Chinese) => "trans.",
        (Term::Ibid, CitationLanguage::Chinese) => "ibid.",
        (Term::Forthcoming, CitationLanguage::Chinese) => "forthcoming",
        (Term::Anonymous, CitationLanguage::Chinese) => "匿名",
        (Term::OpenQuote, CitationLanguage::Chinese) => "\u{300C}",
        (Term::CloseQuote, CitationLanguage::Chinese) => "\u{300D}",
        (Term::OpenInnerQuote, CitationLanguage::Chinese) => "\u{300E}",
        (Term::CloseInnerQuote, CitationLanguage::Chinese) => "\u{300F}",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etal_localization() {
        assert_eq!(term(Term::EtAl, CitationLanguage::English), "et al.");
        assert_eq!(term(Term::EtAl, CitationLanguage::Korean), "외");
        assert_eq!(term(Term::EtAl, CitationLanguage::Japanese), "ほか");
        assert_eq!(term(Term::EtAl, CitationLanguage::Chinese), "等");
    }

    #[test]
    fn test_and_localization() {
        assert_eq!(term(Term::And, CitationLanguage::English), "and");
        assert_eq!(term(Term::And, CitationLanguage::Chinese), "和");
    }

    #[test]
    fn test_no_date_short() {
        assert_eq!(term(Term::NoDateShort, CitationLanguage::English), "n.d.");
        assert_eq!(term(Term::NoDateShort, CitationLanguage::Korean), "n.d.");
    }

    #[test]
    fn test_volume_issue_localization() {
        assert_eq!(term(Term::Volume, CitationLanguage::Japanese), "巻");
        assert_eq!(term(Term::Issue, CitationLanguage::Japanese), "号");
        assert_eq!(term(Term::Volume, CitationLanguage::Chinese), "卷");
        assert_eq!(term(Term::Issue, CitationLanguage::Chinese), "期");
    }

    #[test]
    fn test_all_terms_have_all_languages() {
        let all_terms = [
            Term::EtAl, Term::And, Term::Accessed, Term::In,
            Term::Edition, Term::EditionShort, Term::Volume, Term::VolumeShort,
            Term::Issue, Term::IssueShort, Term::Page, Term::PageShort,
            Term::NoDate, Term::NoDateShort, Term::Editor, Term::EditorShort,
            Term::Translator, Term::TranslatorShort, Term::Ibid, Term::Forthcoming,
            Term::Anonymous, Term::OpenQuote, Term::CloseQuote,
            Term::OpenInnerQuote, Term::CloseInnerQuote,
        ];
        let all_langs = [
            CitationLanguage::English, CitationLanguage::Korean,
            CitationLanguage::Japanese, CitationLanguage::Chinese,
        ];
        for &t in &all_terms {
            for &l in &all_langs {
                let s = term(t, l);
                assert!(!s.is_empty(), "term {:?} for {:?} is empty", t, l);
            }
        }
    }
}
