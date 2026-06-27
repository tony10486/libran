use super::scheme::{ClassificationNode, ClassificationScheme, SchemeCode};

pub struct MscScheme {
    nodes: Vec<ClassificationNode>,
}

impl Default for MscScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl MscScheme {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        for (notation, label, parent) in TOP_LEVEL {
            nodes.push(ClassificationNode {
                id: None,
                scheme_code: SchemeCode::Msc,
                notation: notation.to_string(),
                pref_label: label.to_string(),
                alt_label: None,
                scope_note: None,
                parent_notation: parent.map(|s| s.to_string()),
                sort_order: 0,
            });
        }
        MscScheme { nodes }
    }
}

impl ClassificationScheme for MscScheme {
    fn code(&self) -> SchemeCode {
        SchemeCode::Msc
    }
    fn name(&self) -> &str {
        "Mathematics Subject Classification 2020"
    }
    fn version(&self) -> &str {
        "2020"
    }
    fn license(&self) -> &str {
        "CC BY-NC-SA 4.0"
    }
    fn source_url(&self) -> &str {
        "https://msc2020.org"
    }
    fn is_primary(&self) -> bool {
        false
    }
    fn nodes(&self) -> &[ClassificationNode] {
        &self.nodes
    }
    fn validate_notation(&self, notation: &str) -> bool {
        if notation.is_empty() {
            return false;
        }
        notation.chars().all(|c| {
            c.is_ascii_digit() || c == '-' || c == 'X' || c == 'x' || c.is_ascii_alphabetic()
        })
    }
}

const TOP_LEVEL: &[(&str, &str, Option<&str>)] = &[
    ("00", "General and overarching topics; collections", None),
    ("01", "History and biography", None),
    ("03", "Mathematical logic and foundations", None),
    ("05", "Combinatorics", None),
    ("06", "Order, lattices, ordered algebraic structures", None),
    ("08", "General algebraic systems", None),
    ("11", "Number theory", None),
    ("12", "Field theory and polynomials", None),
    ("13", "Commutative algebra", None),
    ("14", "Algebraic geometry", None),
    ("15", "Linear and multilinear algebra; matrix theory", None),
    ("16", "Associative rings and algebras", None),
    ("17", "Nonassociative rings and algebras", None),
    ("18", "Category theory; homological algebra", None),
    ("19", "K-theory", None),
    ("20", "Group theory and generalizations", None),
    ("22", "Topological groups, Lie groups", None),
    ("26", "Real functions", None),
    ("28", "Measure and integration", None),
    ("30", "Functions of a complex variable", None),
    ("31", "Potential theory", None),
    ("32", "Several complex variables and analytic spaces", None),
    ("33", "Special functions", None),
    ("34", "Ordinary differential equations", None),
    ("35", "Partial differential equations", None),
    ("37", "Dynamical systems and ergodic theory", None),
    ("39", "Difference and functional equations", None),
    ("40", "Sequences, series, summability", None),
    ("41", "Approximations and expansions", None),
    ("42", "Harmonic analysis", None),
    ("43", "Abstract harmonic analysis", None),
    ("44", "Integral transforms, operational calculus", None),
    ("45", "Integral equations", None),
    ("46", "Functional analysis", None),
    ("47", "Operator theory", None),
    (
        "49",
        "Calculus of variations and optimal control; optimization",
        None,
    ),
    ("51", "Geometry", None),
    ("52", "Convex and discrete geometry", None),
    ("53", "Differential geometry", None),
    ("54", "General topology", None),
    ("55", "Algebraic topology", None),
    ("57", "Manifolds and cell complexes", None),
    ("58", "Global analysis, analysis on manifolds", None),
    ("60", "Probability theory and stochastic processes", None),
    ("62", "Statistics", None),
    ("65", "Numerical analysis", None),
    ("68", "Computer science", None),
    ("70", "Mechanics of particles and systems", None),
    ("74", "Mechanics of deformable solids", None),
    ("76", "Fluid mechanics", None),
    ("78", "Optics, electromagnetic theory", None),
    ("80", "Classical thermodynamics, heat transfer", None),
    ("81", "Quantum theory", None),
    ("82", "Statistical mechanics, structure of matter", None),
    ("83", "Relativity and gravitational theory", None),
    ("85", "Astronomy and astrophysics", None),
    ("86", "Geophysics", None),
    ("90", "Operations research, mathematical programming", None),
    (
        "91",
        "Game theory, economics, social and behavioral sciences",
        None,
    ),
    ("92", "Biology and other natural sciences", None),
    ("94", "Information and communication, circuits", None),
    ("97", "Mathematics education", None),
];
