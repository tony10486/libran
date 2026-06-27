use super::scheme::{ClassificationNode, ClassificationScheme, SchemeCode};

pub struct UdcScheme {
    nodes: Vec<ClassificationNode>,
}

impl Default for UdcScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl UdcScheme {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        for (notation, label, parent) in TOP_LEVEL {
            nodes.push(ClassificationNode {
                id: None,
                scheme_code: SchemeCode::Udc,
                notation: notation.to_string(),
                pref_label: label.to_string(),
                alt_label: None,
                scope_note: None,
                parent_notation: parent.map(|s| s.to_string()),
                sort_order: 0,
            });
        }
        UdcScheme { nodes }
    }

    pub fn from_nodes(nodes: Vec<ClassificationNode>) -> Self {
        UdcScheme { nodes }
    }
}

impl ClassificationScheme for UdcScheme {
    fn code(&self) -> SchemeCode {
        SchemeCode::Udc
    }
    fn name(&self) -> &str {
        "Universal Decimal Classification Summary"
    }
    fn version(&self) -> &str {
        "MRF 2011 / Finto 2020-11-25"
    }
    fn license(&self) -> &str {
        "CC BY-SA 3.0"
    }
    fn source_url(&self) -> &str {
        "https://finto.fi/rest/v1/udcs/data?format=text/turtle"
    }
    fn is_primary(&self) -> bool {
        true
    }
    fn nodes(&self) -> &[ClassificationNode] {
        &self.nodes
    }
    fn validate_notation(&self, notation: &str) -> bool {
        if notation.is_empty() {
            return false;
        }
        for c in notation.chars() {
            if !c.is_ascii_digit()
                && c != '.'
                && c != '-'
                && c != '+'
                && c != '/'
                && c != ':'
                && c != '('
                && c != ')'
                && c != '='
                && c != ' '
                && c != '\''
                && !c.is_ascii_alphabetic()
            {
                return false;
            }
        }
        true
    }
}

const TOP_LEVEL: &[(&str, &str, Option<&str>)] = &[
    ("0", "Computer science, information & general works", None),
    ("00", "Knowledge & scholarship", Some("0")),
    ("004", "Computer science & computing", Some("0")),
    ("005", "Software engineering", Some("004")),
    ("01", "Bibliography & bibliographies", Some("0")),
    ("02", "Librarianship", Some("0")),
    ("1", "Philosophy & psychology", None),
    ("11", "Epistemology & causation", Some("1")),
    ("15", "Non-naturalistic philosophies", Some("1")),
    ("16", "Logic", Some("1")),
    ("17", "Ethics & moral philosophy", Some("1")),
    ("2", "Religion", None),
    ("21", "Pre-Christian & early Christian religions", Some("2")),
    ("22", "Bible", Some("2")),
    ("23", "Christian denominations", Some("2")),
    ("24", "Christian practice & religious life", Some("2")),
    ("25", "Pastoral & spiritual care", Some("2")),
    ("26", "Christian church organization", Some("2")),
    ("27", "History of Christianity", Some("2")),
    ("28", "Christian sects", Some("2")),
    ("29", "Other religions", Some("2")),
    ("3", "Social sciences", None),
    ("31", "Statistics & demography", Some("3")),
    ("32", "Politics & political science", Some("3")),
    ("33", "Economics", Some("3")),
    ("34", "Law", Some("3")),
    ("35", "Public administration", Some("3")),
    ("36", "Social welfare & social problems", Some("3")),
    ("37", "Education", Some("3")),
    ("38", "Commerce & communications", Some("3")),
    ("39", "Customs & folklore", Some("3")),
    ("4", "(Vacant)", None),
    ("5", "Mathematics & natural sciences", None),
    ("50", "General natural sciences", Some("5")),
    ("51", "Mathematics", Some("5")),
    ("512", "Algebra", Some("51")),
    ("514", "Geometry", Some("51")),
    ("515", "Topology", Some("51")),
    ("517", "Analysis", Some("51")),
    (
        "517.9",
        "Differential equations & integral equations",
        Some("517"),
    ),
    ("52", "Astronomy & astrophysics", Some("5")),
    ("53", "Physics", Some("5")),
    ("531", "Mechanics", Some("53")),
    ("532", "Fluid mechanics", Some("53")),
    ("533", "Gas mechanics", Some("53")),
    ("535", "Optics", Some("53")),
    ("537", "Electricity & electromagnetism", Some("53")),
    ("538", "Magnetism", Some("53")),
    ("539", "Physical nature of matter", Some("53")),
    ("54", "Chemistry", Some("5")),
    ("541", "Physical chemistry", Some("54")),
    ("542", "Practical & laboratory chemistry", Some("54")),
    ("543", "Analytical chemistry", Some("54")),
    ("546", "Inorganic chemistry", Some("54")),
    ("547", "Organic chemistry", Some("54")),
    ("55", "Earth sciences & geology", Some("5")),
    ("56", "Paleontology", Some("5")),
    ("57", "Biology", Some("5")),
    ("58", "Botany", Some("5")),
    ("59", "Zoology", Some("5")),
    ("6", "Applied sciences & technology", None),
    ("60", "General applied sciences", Some("6")),
    ("61", "Medicine & health", Some("6")),
    ("62", "Engineering", Some("6")),
    ("63", "Agriculture & related technologies", Some("6")),
    ("64", "Home economics", Some("6")),
    ("65", "Management & business", Some("6")),
    ("66", "Chemical technology", Some("6")),
    ("67", "Manufacturing & metalworking", Some("6")),
    ("68", "Other technologies", Some("6")),
    ("69", "Building & construction", Some("6")),
    ("7", "Arts & recreation", None),
    ("71", "Physical planning & urban design", Some("7")),
    ("72", "Architecture", Some("7")),
    ("73", "Sculpture", Some("7")),
    ("74", "Drawing & decorative arts", Some("7")),
    ("75", "Painting", Some("7")),
    ("76", "Graphic arts & prints", Some("7")),
    ("77", "Photography", Some("7")),
    ("78", "Music", Some("7")),
    ("79", "Recreation & sports", Some("7")),
    ("8", "Language & linguistics", None),
    ("80", "General linguistics", Some("8")),
    ("81", "Languages", Some("8")),
    ("82", "Literature", Some("8")),
    ("9", "History & geography", None),
    ("90", "General history", Some("9")),
    ("91", "Geography", Some("9")),
    ("92", "Biography", Some("9")),
    ("93", "History of ancient world", Some("9")),
    ("94", "History of modern world", Some("9")),
];
