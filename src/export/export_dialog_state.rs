use crate::citation::text::engine::render_citation;
use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
use crate::db::documents::Document;
use crate::export::ExportFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogSection {
    Format,
    Style,
    Language,
    DisplayMode,
    Preview,
}

#[derive(Clone, Debug)]
pub struct ExportDialogState {
    pub selected_format: ExportFormat,
    pub selected_style: CitationStyle,
    pub selected_language: CitationLanguage,
    pub display_mode: DisplayMode,
    pub focused_section: DialogSection,
    pub format_cursor: usize,
    pub style_cursor: usize,
    pub language_cursor: usize,
    pub display_mode_cursor: usize,
    pub preview_text: String,
}

impl ExportDialogState {
    pub fn new() -> Self {
        Self {
            selected_format: ExportFormat::Bibtex,
            selected_style: CitationStyle::Apa7th,
            selected_language: CitationLanguage::English,
            display_mode: DisplayMode::InText,
            focused_section: DialogSection::Format,
            format_cursor: 0,
            style_cursor: 0,
            language_cursor: 0,
            display_mode_cursor: 0,
            preview_text: String::new(),
        }
    }

    fn active_sections(&self) -> Vec<DialogSection> {
        if self.is_display_mode_active() {
            vec![
                DialogSection::Format,
                DialogSection::Style,
                DialogSection::Language,
                DialogSection::DisplayMode,
                DialogSection::Preview,
            ]
        } else {
            vec![
                DialogSection::Format,
                DialogSection::Style,
                DialogSection::Language,
                DialogSection::Preview,
            ]
        }
    }

    pub fn tab_next(&mut self) {
        let sections = self.active_sections();
        if let Some(pos) = sections.iter().position(|s| *s == self.focused_section) {
            let next = (pos + 1) % sections.len();
            self.focused_section = sections[next];
        } else {
            self.focused_section = sections[0];
        }
    }

    pub fn tab_prev(&mut self) {
        let sections = self.active_sections();
        if let Some(pos) = sections.iter().position(|s| *s == self.focused_section) {
            let prev = if pos == 0 {
                sections.len() - 1
            } else {
                pos - 1
            };
            self.focused_section = sections[prev];
        } else {
            self.focused_section = sections[sections.len() - 1];
        }
    }

    pub fn cursor_down(&mut self) {
        match self.focused_section {
            DialogSection::Format => self.move_format_cursor(1),
            DialogSection::Style => self.move_style_cursor(1),
            DialogSection::Language => self.move_language_cursor(1),
            DialogSection::DisplayMode => self.move_display_mode_cursor(1),
            DialogSection::Preview => {}
        }
    }

    pub fn cursor_up(&mut self) {
        match self.focused_section {
            DialogSection::Format => self.move_format_cursor(-1),
            DialogSection::Style => self.move_style_cursor(-1),
            DialogSection::Language => self.move_language_cursor(-1),
            DialogSection::DisplayMode => self.move_display_mode_cursor(-1),
            DialogSection::Preview => {}
        }
    }

    fn move_format_cursor(&mut self, direction: i32) {
        let implemented: Vec<(usize, ExportFormat)> = ExportFormat::all()
            .iter()
            .enumerate()
            .filter(|(_, f)| f.is_implemented())
            .map(|(i, f)| (i, f.clone()))
            .collect();
        if implemented.is_empty() {
            return;
        }
        let current_pos = implemented
            .iter()
            .position(|(_, f)| *f == self.selected_format)
            .unwrap_or(0);
        let new_pos = if direction > 0 {
            (current_pos + 1) % implemented.len()
        } else if current_pos == 0 {
            implemented.len() - 1
        } else {
            current_pos - 1
        };
        self.format_cursor = implemented[new_pos].0;
        self.selected_format = implemented[new_pos].1.clone();
    }

    fn move_style_cursor(&mut self, direction: i32) {
        let implemented: Vec<(usize, CitationStyle)> = CitationStyle::all()
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_implemented())
            .map(|(i, s)| (i, *s))
            .collect();
        if implemented.is_empty() {
            return;
        }
        let current_pos = implemented
            .iter()
            .position(|(_, s)| *s == self.selected_style)
            .unwrap_or(0);
        let new_pos = if direction > 0 {
            (current_pos + 1) % implemented.len()
        } else if current_pos == 0 {
            implemented.len() - 1
        } else {
            current_pos - 1
        };
        self.style_cursor = implemented[new_pos].0;
        self.selected_style = implemented[new_pos].1;

        if !self.is_display_mode_active() && self.focused_section == DialogSection::DisplayMode {
            self.tab_next();
        }
    }

    fn move_language_cursor(&mut self, direction: i32) {
        let all = CitationLanguage::all();
        let current_pos = all
            .iter()
            .position(|l| *l == self.selected_language)
            .unwrap_or(0);
        let new_pos = if direction > 0 {
            (current_pos + 1) % all.len()
        } else if current_pos == 0 {
            all.len() - 1
        } else {
            current_pos - 1
        };
        self.language_cursor = new_pos;
        self.selected_language = all[new_pos];
    }

    fn move_display_mode_cursor(&mut self, direction: i32) {
        let modes = [
            DisplayMode::InText,
            DisplayMode::Footnotes,
            DisplayMode::Endnotes,
        ];
        let current_pos = modes
            .iter()
            .position(|m| *m == self.display_mode)
            .unwrap_or(0);
        let new_pos = if direction > 0 {
            (current_pos + 1) % modes.len()
        } else if current_pos == 0 {
            modes.len() - 1
        } else {
            current_pos - 1
        };
        self.display_mode_cursor = new_pos;
        self.display_mode = modes[new_pos];
    }

    pub fn is_display_mode_active(&self) -> bool {
        self.selected_style.is_notes_based()
    }

    pub fn update_preview(&mut self, doc: &Document) {
        let result = render_citation(
            doc,
            self.selected_style,
            self.selected_language,
            self.display_mode,
        );
        self.preview_text =
            result.unwrap_or_else(|e| format!("[{}] {}", self.selected_style.display_name(), e));
    }
}

impl Default for ExportDialogState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc() -> Document {
        Document {
            id: Some(1),
            title: "Deep learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            conference: None,
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            arxiv_id: None,
            abstract_text: None,
            keywords: None,
            file_path: None,
            file_hash: None,
            citation_key: None,
            source: None,
            rating: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_tab_cycles_through_sections() {
        let mut state = ExportDialogState::new();
        assert_eq!(state.focused_section, DialogSection::Format);
        state.tab_next();
        assert_eq!(state.focused_section, DialogSection::Style);
        state.tab_next();
        assert_eq!(state.focused_section, DialogSection::Language);
        state.tab_next();
        assert_eq!(state.focused_section, DialogSection::Preview);
        state.tab_next();
        assert_eq!(state.focused_section, DialogSection::Format);
    }

    #[test]
    fn test_tab_prev_cycles_backwards() {
        let mut state = ExportDialogState::new();
        state.tab_prev();
        assert_eq!(state.focused_section, DialogSection::Preview);
        state.tab_prev();
        assert_eq!(state.focused_section, DialogSection::Language);
    }

    #[test]
    fn test_format_cursor_cycles_through_all() {
        let mut state = ExportDialogState::new();
        assert_eq!(state.selected_format, ExportFormat::Bibtex);
        state.cursor_down();
        assert_eq!(state.selected_format, ExportFormat::Bookmarks);
        state.cursor_down();
        assert_eq!(state.selected_format, ExportFormat::Cff);
        state.cursor_up();
        assert_eq!(state.selected_format, ExportFormat::Bookmarks);
        state.cursor_up();
        assert_eq!(state.selected_format, ExportFormat::Bibtex);
    }

    #[test]
    fn test_style_cursor_cycles_all_styles() {
        let mut state = ExportDialogState::new();
        state.focused_section = DialogSection::Style;
        assert_eq!(state.selected_style, CitationStyle::Apa7th);
        state.cursor_down();
        assert_ne!(state.selected_style, CitationStyle::Apa7th);
        state.cursor_up();
        assert_eq!(state.selected_style, CitationStyle::Apa7th);
    }

    #[test]
    fn test_language_cursor_wraps() {
        let mut state = ExportDialogState::new();
        state.focused_section = DialogSection::Language;
        assert_eq!(state.selected_language, CitationLanguage::English);
        state.cursor_down();
        assert_eq!(state.selected_language, CitationLanguage::Korean);
        state.cursor_down();
        assert_eq!(state.selected_language, CitationLanguage::Japanese);
        state.cursor_down();
        assert_eq!(state.selected_language, CitationLanguage::Chinese);
        state.cursor_down();
        assert_eq!(state.selected_language, CitationLanguage::English);
    }

    #[test]
    fn test_update_preview_renders_apa() {
        let mut state = ExportDialogState::new();
        state.update_preview(&make_doc());
        assert!(state.preview_text.contains("Smith, J. (2023)"));
        assert!(state.preview_text.contains("Deep learning"));
    }

    #[test]
    fn test_display_mode_not_active_for_apa() {
        let state = ExportDialogState::new();
        assert!(!state.is_display_mode_active());
    }
}
