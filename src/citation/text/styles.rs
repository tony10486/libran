/// Supported citation styles.
///
/// Only [`CitationStyle::Apa7th`] is implemented in Phase 1. All other
/// variants are defined so the UI can list and select them, but rendering
/// them bails with a "not yet implemented" error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CitationStyle {
    AcsGuide2022,
    Ama11th,
    Apa7th,
    Apsa2018,
    Asa6th7th,
    Chicago18AuthorDate,
    Chicago18NotesBib,
    Chicago18ShortenedNotesBib,
    CiteThemRight12thHarvard,
    ElsevierHarvardWithTitles,
    IeeeV11_29_2023,
    Mhra4thNotes,
    Mla9thInText,
    Nature,
    NlmVancouverCitingMedicine2nd,
}

impl CitationStyle {
    /// Human-readable name suitable for UI display.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AcsGuide2022 => "ACS Guide 2022",
            Self::Ama11th => "AMA 11th",
            Self::Apa7th => "APA 7th",
            Self::Apsa2018 => "APSA 2018",
            Self::Asa6th7th => "ASA 6th/7th",
            Self::Chicago18AuthorDate => "Chicago 18th (author-date)",
            Self::Chicago18NotesBib => "Chicago 18th (notes+bib)",
            Self::Chicago18ShortenedNotesBib => "Chicago 18th (shortened notes+bib)",
            Self::CiteThemRight12thHarvard => "Cite Them Right 12th Harvard",
            Self::ElsevierHarvardWithTitles => "Elsevier Harvard with titles",
            Self::IeeeV11_29_2023 => "IEEE v11.29.2023",
            Self::Mhra4thNotes => "MHRA 4th notes",
            Self::Mla9thInText => "MLA 9th in-text",
            Self::Nature => "Nature",
            Self::NlmVancouverCitingMedicine2nd => "NLM/Vancouver Citing Medicine 2nd",
        }
    }

    /// All 15 styles in the canonical declaration order.
    pub fn all() -> &'static [CitationStyle] {
        const ALL: &[CitationStyle] = &[
            CitationStyle::AcsGuide2022,
            CitationStyle::Ama11th,
            CitationStyle::Apa7th,
            CitationStyle::Apsa2018,
            CitationStyle::Asa6th7th,
            CitationStyle::Chicago18AuthorDate,
            CitationStyle::Chicago18NotesBib,
            CitationStyle::Chicago18ShortenedNotesBib,
            CitationStyle::CiteThemRight12thHarvard,
            CitationStyle::ElsevierHarvardWithTitles,
            CitationStyle::IeeeV11_29_2023,
            CitationStyle::Mhra4thNotes,
            CitationStyle::Mla9thInText,
            CitationStyle::Nature,
            CitationStyle::NlmVancouverCitingMedicine2nd,
        ];
        ALL
    }

    /// Whether the renderer has a working implementation for this style.
    /// Phase 1 implements only APA 7th.
    pub fn is_implemented(&self) -> bool {
        matches!(self, Self::Apa7th)
    }

    /// Whether the style uses footnotes/endnotes rather than in-text citations.
    pub fn is_notes_based(&self) -> bool {
        matches!(
            self,
            Self::Chicago18NotesBib
                | Self::Chicago18ShortenedNotesBib
                | Self::Mhra4thNotes
        )
    }
}

/// Output language for rendered citations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CitationLanguage {
    English,
    Korean,
    Japanese,
    Chinese,
}

impl CitationLanguage {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Korean => "한국어",
            Self::Japanese => "日本語",
            Self::Chinese => "中文",
        }
    }

    pub fn all() -> &'static [CitationLanguage] {
        const ALL: &[CitationLanguage] = &[
            CitationLanguage::English,
            CitationLanguage::Korean,
            CitationLanguage::Japanese,
            CitationLanguage::Chinese,
        ];
        ALL
    }
}

/// Where the citation will be displayed, which affects formatting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    InText,
    Footnotes,
    Endnotes,
}

impl DisplayMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::InText => "In-text",
            Self::Footnotes => "Footnotes",
            Self::Endnotes => "Endnotes",
        }
    }

    /// Whether this display mode is applicable for the given style.
    /// Non-notes styles only support [`DisplayMode::InText`].
    pub fn is_applicable(self, style: CitationStyle) -> bool {
        if style.is_notes_based() {
            true
        } else {
            matches!(self, Self::InText)
        }
    }
}
