use super::scheme::{ClassificationNode, ClassificationScheme, SchemeCode};

pub struct PhyshScheme {
    nodes: Vec<ClassificationNode>,
}

impl Default for PhyshScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl PhyshScheme {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        for (notation, label, parent) in TOP_LEVEL {
            nodes.push(ClassificationNode {
                id: None,
                scheme_code: SchemeCode::Physh,
                notation: notation.to_string(),
                pref_label: label.to_string(),
                alt_label: None,
                scope_note: None,
                parent_notation: parent.map(|s| s.to_string()),
                sort_order: 0,
            });
        }
        PhyshScheme { nodes }
    }
}

impl ClassificationScheme for PhyshScheme {
    fn code(&self) -> SchemeCode {
        SchemeCode::Physh
    }
    fn name(&self) -> &str {
        "Physics Subject Headings"
    }
    fn version(&self) -> &str {
        "v2.8.0"
    }
    fn license(&self) -> &str {
        "CC0 1.0"
    }
    fn source_url(&self) -> &str {
        "https://github.com/physh-org/PhySH"
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

const TOP_LEVEL: &[(&str, &str, Option<&str>)] = &[
    ("Research Areas", "Research Areas", None),
    (
        "Condensed Matter",
        "Condensed Matter Physics",
        Some("Research Areas"),
    ),
    (
        "Atomic, Molecular & Optical",
        "Atomic, Molecular & Optical Physics",
        Some("Research Areas"),
    ),
    (
        "Particles & Fields",
        "Particles & Fields",
        Some("Research Areas"),
    ),
    ("Nuclear Physics", "Nuclear Physics", Some("Research Areas")),
    (
        "Biological Physics",
        "Biological Physics",
        Some("Research Areas"),
    ),
    ("Geophysics", "Geophysics", Some("Research Areas")),
    ("Astrophysics", "Astrophysics", Some("Research Areas")),
    ("Plasma Physics", "Plasma Physics", Some("Research Areas")),
    (
        "Classical Physics",
        "Classical Physics",
        Some("Research Areas"),
    ),
    (
        "Quantum Information",
        "Quantum Information",
        Some("Research Areas"),
    ),
    ("Physical Systems", "Physical Systems", None),
    ("Properties", "Properties", None),
    ("Techniques", "Techniques", None),
    ("Professional Topics", "Professional Topics", None),
];
