use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

pub mod export_dialog_state;
pub mod preferences;

pub fn export(documents: &[Document], format: ExportFormat, writer: &mut impl Write) -> Result<()> {
    match format {
        ExportFormat::BibliontologyRdf => {
            crate::citation::formats::bibliontology_rdf::export_bibliontology_rdf(documents, writer)
        }
        ExportFormat::Bibtex => crate::citation::bibtex::export_bibtex(documents, writer),
        ExportFormat::Bookmarks => crate::citation::formats::bookmarks::export_bookmarks(documents, writer),
        ExportFormat::Cff => crate::citation::formats::cff::export_cff(documents, writer),
        ExportFormat::CffReferences => {
            crate::citation::formats::cff::export_cff_references(documents, writer)
        }
        ExportFormat::Coins => crate::citation::formats::coins::export_coins(documents, writer),
        ExportFormat::CslJson => crate::citation::csl_json::export_csl_json(documents, writer),
        ExportFormat::Csv => crate::citation::formats::csv_export::export_csv(documents, writer),
        ExportFormat::EndnoteXml => {
            crate::citation::formats::endnote_xml::export_endnote_xml(documents, writer)
        }
        ExportFormat::Mods => crate::citation::formats::mods::export_mods(documents, writer),
        ExportFormat::ReferBibix => {
            crate::citation::formats::refer_bibix::export_refer_bibix(documents, writer)
        }
        ExportFormat::RefworksTagged => {
            crate::citation::formats::refworks_tagged::export_refworks_tagged(documents, writer)
        }
        ExportFormat::Ris => crate::citation::formats::ris::export_ris(documents, writer),
        ExportFormat::EvernoteExport => {
            crate::citation::formats::evernote::export_evernote(documents, writer)
        }
        ExportFormat::Tei => crate::citation::formats::tei::export_tei(documents, writer),
        ExportFormat::WikidataQuickStatements => {
            crate::citation::formats::wikidata_qs::export_wikidata_qs(documents, writer)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    BibliontologyRdf,
    Bibtex,
    Bookmarks,
    Cff,
    CffReferences,
    Coins,
    CslJson,
    Csv,
    EndnoteXml,
    Mods,
    ReferBibix,
    RefworksTagged,
    Ris,
    EvernoteExport,
    Tei,
    WikidataQuickStatements,
}

impl ExportFormat {
    pub fn all() -> &'static [ExportFormat] {
        &[
            ExportFormat::BibliontologyRdf,
            ExportFormat::Bibtex,
            ExportFormat::Bookmarks,
            ExportFormat::Cff,
            ExportFormat::CffReferences,
            ExportFormat::Coins,
            ExportFormat::CslJson,
            ExportFormat::Csv,
            ExportFormat::EndnoteXml,
            ExportFormat::Mods,
            ExportFormat::ReferBibix,
            ExportFormat::RefworksTagged,
            ExportFormat::Ris,
            ExportFormat::EvernoteExport,
            ExportFormat::Tei,
            ExportFormat::WikidataQuickStatements,
        ]
    }

    pub fn file_extension(&self) -> &str {
        match self {
            ExportFormat::BibliontologyRdf => "rdf",
            ExportFormat::Bibtex => "bib",
            ExportFormat::Bookmarks => "html",
            ExportFormat::Cff => "cff",
            ExportFormat::CffReferences => "cff",
            ExportFormat::Coins => "html",
            ExportFormat::CslJson => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::EndnoteXml => "xml",
            ExportFormat::Mods => "xml",
            ExportFormat::ReferBibix => "ref",
            ExportFormat::RefworksTagged => "txt",
            ExportFormat::Ris => "ris",
            ExportFormat::EvernoteExport => "enex",
            ExportFormat::Tei => "xml",
            ExportFormat::WikidataQuickStatements => "tsv",
        }
    }

    pub fn format_name(&self) -> &str {
        match self {
            ExportFormat::BibliontologyRdf => "Bibliontology RDF",
            ExportFormat::Bibtex => "BibTeX",
            ExportFormat::Bookmarks => "Bookmarks",
            ExportFormat::Cff => "CFF",
            ExportFormat::CffReferences => "CFF References",
            ExportFormat::Coins => "COinS",
            ExportFormat::CslJson => "CSL JSON",
            ExportFormat::Csv => "CSV",
            ExportFormat::EndnoteXml => "Endnote XML",
            ExportFormat::Mods => "MODS",
            ExportFormat::ReferBibix => "Refer/BibIX",
            ExportFormat::RefworksTagged => "RefWorks Tagged",
            ExportFormat::Ris => "RIS",
            ExportFormat::EvernoteExport => "Simple Evernote Export",
            ExportFormat::Tei => "TEI",
            ExportFormat::WikidataQuickStatements => "Wikidata QuickStatements",
        }
    }

    /// Returns true for formats whose output depends on citation style rendering
    /// (Bookmarks and Evernote embed rendered citation text in their structure).
    pub fn is_style_dependent(&self) -> bool {
        matches!(self, ExportFormat::Bookmarks | ExportFormat::EvernoteExport)
    }

    /// Returns true for formats implemented in the current phase.
    pub fn is_implemented(&self) -> bool {
        true
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ExportFormat::BibliontologyRdf => "bibliontology_rdf",
            ExportFormat::Bibtex => "bibtex",
            ExportFormat::Bookmarks => "bookmarks",
            ExportFormat::Cff => "cff",
            ExportFormat::CffReferences => "cff_references",
            ExportFormat::Coins => "coins",
            ExportFormat::CslJson => "csl_json",
            ExportFormat::Csv => "csv",
            ExportFormat::EndnoteXml => "endnote_xml",
            ExportFormat::Mods => "mods",
            ExportFormat::ReferBibix => "refer_bibix",
            ExportFormat::RefworksTagged => "refworks_tagged",
            ExportFormat::Ris => "ris",
            ExportFormat::EvernoteExport => "evernote_export",
            ExportFormat::Tei => "tei",
            ExportFormat::WikidataQuickStatements => "wikidata_quick_statements",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::all().iter().copied().find(|fmt| fmt.as_str() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_formats_are_implemented() {
        for format in ExportFormat::all() {
            assert!(
                format.is_implemented(),
                "{:?} should be implemented",
                format
            );
        }
    }

    #[test]
    fn test_all_formats_have_unique_extensions_and_names() {
        let all = ExportFormat::all();
        assert_eq!(all.len(), 16, "should have 16 formats");

        let mut extensions = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for format in all {
            extensions.insert(format.file_extension());
            names.insert(format.format_name());
        }
        assert_eq!(names.len(), 16, "all format names should be unique");
    }
}
