use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

pub mod export_dialog_state;

pub fn export(documents: &[Document], format: ExportFormat, writer: &mut impl Write) -> Result<()> {
    match format {
        ExportFormat::Bibtex => crate::citation::bibtex::export_bibtex(documents, writer),
        ExportFormat::CslJson => crate::citation::csl_json::export_csl_json(documents, writer),
        ExportFormat::Ris => crate::citation::formats::ris::export_ris(documents, writer),
        ExportFormat::Csv => crate::citation::formats::csv_export::export_csv(documents, writer),
        ExportFormat::Mods => crate::citation::formats::mods::export_mods(documents, writer),
        _ => anyhow::bail!("{} format not yet implemented", format.format_name()),
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
        matches!(
            self,
            ExportFormat::Bibtex
                | ExportFormat::CslJson
                | ExportFormat::Ris
                | ExportFormat::Csv
                | ExportFormat::Mods
        )
    }
}
