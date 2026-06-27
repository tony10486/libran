use super::scheme::{ClassificationNode, ClassificationScheme, SchemeCode};

pub struct LccScheme {
    nodes: Vec<ClassificationNode>,
}

impl Default for LccScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl LccScheme {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        for (notation, label) in TOP_LEVEL {
            nodes.push(ClassificationNode {
                id: None,
                scheme_code: SchemeCode::Lcc,
                notation: notation.to_string(),
                pref_label: label.to_string(),
                alt_label: None,
                scope_note: None,
                parent_notation: None,
                sort_order: 0,
            });
        }
        LccScheme { nodes }
    }
}

impl ClassificationScheme for LccScheme {
    fn code(&self) -> SchemeCode {
        SchemeCode::Lcc
    }
    fn name(&self) -> &str {
        "Library of Congress Classification"
    }
    fn version(&self) -> &str {
        "2024"
    }
    fn license(&self) -> &str {
        "Public Domain (US Government Work)"
    }
    fn source_url(&self) -> &str {
        "https://id.loc.gov/authorities/classification"
    }
    fn is_primary(&self) -> bool {
        false
    }
    fn nodes(&self) -> &[ClassificationNode] {
        &self.nodes
    }
    fn validate_notation(&self, notation: &str) -> bool {
        !notation.is_empty()
    }
}

const TOP_LEVEL: &[(&str, &str)] = &[
    ("A", "General Works"),
    ("B", "Philosophy, Psychology, Religion"),
    ("C", "Auxiliary Sciences of History"),
    (
        "D",
        "World History and History of Europe, Asia, Africa, Australia, New Zealand, etc.",
    ),
    ("E", "History of the Americas"),
    ("F", "History of the Americas"),
    ("G", "Geography, Anthropology, Recreation"),
    ("H", "Social Sciences"),
    ("J", "Political Science"),
    ("K", "Law"),
    ("L", "Education"),
    ("M", "Music"),
    ("N", "Fine Arts"),
    ("P", "Language and Literature"),
    ("Q", "Science"),
    ("R", "Medicine"),
    ("S", "Agriculture"),
    ("T", "Technology"),
    ("U", "Military Science"),
    ("V", "Naval Science"),
    ("Z", "Bibliography, Library Science, Information Resources"),
];
