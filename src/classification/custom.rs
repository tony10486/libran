use super::scheme::{ClassificationNode, ClassificationScheme, SchemeCode};

pub struct CustomScheme {
    code: String,
    name: String,
    nodes: Vec<ClassificationNode>,
}

impl CustomScheme {
    pub fn new(code: &str, name: &str, nodes: Vec<ClassificationNode>) -> Self {
        CustomScheme {
            code: code.to_string(),
            name: name.to_string(),
            nodes,
        }
    }
}

impl ClassificationScheme for CustomScheme {
    fn code(&self) -> SchemeCode {
        SchemeCode::Custom(self.code.clone())
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn version(&self) -> &str {
        "user-defined"
    }
    fn license(&self) -> &str {
        "user-owned"
    }
    fn source_url(&self) -> &str {
        ""
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
