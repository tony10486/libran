use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

pub fn export(documents: &[Document], format: ExportFormat, writer: &mut impl Write) -> Result<()> {
    match format {
        ExportFormat::Bibtex => crate::citation::bibtex::export_bibtex(documents, writer),
        ExportFormat::CslJson => crate::citation::csl_json::export_csl_json(documents, writer),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportFormat {
    Bibtex,
    CslJson,
}
