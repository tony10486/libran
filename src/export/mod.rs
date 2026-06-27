use crate::config::ExportFormatConfig;
use crate::db::documents::Document;
use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

pub mod export_dialog_state;
pub mod preferences;

static CUSTOM_FORMATS: Mutex<Option<HashMap<String, ExportFormatConfig>>> = Mutex::new(None);

// ── User data helpers ──

/// A single classification assigned to a document.
#[derive(Clone, Debug)]
pub struct DocClassification {
    pub scheme: String,
    pub notation: String,
    pub label: String,
}

/// User-created data associated with a document: notes, tags, classifications,
/// projects, and custom fields. Fetched from the DB and passed to
/// `_with_user_data` export variants.
#[derive(Clone, Debug, Default)]
pub struct DocUserData {
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub classifications: Vec<DocClassification>,
    pub projects: Vec<String>,
    pub custom_fields: Vec<(String, String)>,
}

/// Fetch all user-created data for a single document.
pub fn fetch_user_data(conn: &Connection, doc_id: i64) -> Result<DocUserData> {
    let notes: Option<String> = {
        let notes_vec = crate::db::notes::list(conn, doc_id)?
            .into_iter()
            .map(|n| n.content)
            .collect::<Vec<_>>();
        if notes_vec.is_empty() {
            None
        } else {
            Some(notes_vec.join("\n\n"))
        }
    };
    let tags = crate::db::documents::get_tags(conn, doc_id)?;

    let mut stmt = conn.prepare(
        "SELECT cs.code, cn.notation, cn.pref_label
         FROM document_classifications dc
         INNER JOIN classification_nodes cn ON dc.node_id = cn.id
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         WHERE dc.document_id = ?1
         ORDER BY cs.code, cn.notation",
    )?;
    let classifications: Vec<DocClassification> = stmt
        .query_map(params![doc_id], |row| {
            Ok(DocClassification {
                scheme: row.get(0)?,
                notation: row.get(1)?,
                label: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn.prepare(
        "SELECT p.name FROM projects p
         INNER JOIN project_documents pd ON p.id = pd.project_id
         WHERE pd.document_id = ?1
         ORDER BY p.name",
    )?;
    let projects: Vec<String> = stmt
        .query_map(params![doc_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let fields = crate::db::custom_fields::list_fields(conn, doc_id)?;
    let custom_fields: Vec<(String, String)> =
        fields.into_iter().map(|f| (f.key, f.value)).collect();

    Ok(DocUserData {
        notes,
        tags,
        classifications,
        projects,
        custom_fields,
    })
}

fn fetch_projects_for_doc(conn: &Connection, doc_id: i64) -> Vec<String> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT p.name FROM projects p
         INNER JOIN project_documents pd ON p.id = pd.project_id
         WHERE pd.document_id = ?1
         ORDER BY p.name",
    ) else {
        return Vec::new();
    };
    let rows = match stmt.query_map(rusqlite::params![doc_id], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    rows.filter_map(|r| r.ok()).collect()
}

// ── Full library JSON export ──

#[derive(Serialize)]
struct FullLibraryExport {
    documents: Vec<FullDocumentExport>,
}

#[derive(Serialize)]
struct FullDocumentExport {
    #[serde(flatten)]
    document: Document,
    notes: Vec<FullNoteExport>,
    tags: Vec<String>,
    classifications: Vec<FullClassificationExport>,
    projects: Vec<String>,
    custom_fields: Vec<FullCustomFieldExport>,
}

#[derive(Serialize)]
struct FullNoteExport {
    content: String,
    note_type: String,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Serialize)]
struct FullClassificationExport {
    scheme: String,
    notation: String,
    is_primary: bool,
}

#[derive(Serialize)]
struct FullCustomFieldExport {
    key: String,
    value: String,
}

/// Export the entire library (all documents + all related user data) as JSON.
pub fn export_full_library_json(conn: &Connection, writer: &mut impl Write) -> Result<()> {
    let docs = crate::db::documents::list_all(conn)?;

    let full_docs = docs
        .into_iter()
        .map(|doc| {
            let doc_id = doc.id.unwrap_or(0);
            let notes = crate::db::notes::list(conn, doc_id)
                .unwrap_or_default()
                .into_iter()
                .map(|n| FullNoteExport {
                    content: n.content,
                    note_type: n.note_type,
                    created_at: n.created_at,
                    updated_at: n.updated_at,
                })
                .collect();

            let tags = crate::db::documents::get_tags(conn, doc_id).unwrap_or_default();

            let classifications = fetch_full_classifications(conn, doc_id);

            let projects = fetch_projects_for_doc(conn, doc_id);

            let custom_fields = crate::db::custom_fields::list_fields(conn, doc_id)
                .unwrap_or_default()
                .into_iter()
                .map(|f| FullCustomFieldExport {
                    key: f.key,
                    value: f.value,
                })
                .collect();

            FullDocumentExport {
                document: doc,
                notes,
                tags,
                classifications,
                projects,
                custom_fields,
            }
        })
        .collect();

    let library = FullLibraryExport {
        documents: full_docs,
    };
    let json = serde_json::to_string_pretty(&library)?;
    writer.write_all(json.as_bytes())?;
    Ok(())
}

fn fetch_full_classifications(conn: &Connection, doc_id: i64) -> Vec<FullClassificationExport> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT cs.code, cn.notation, dc.is_primary
         FROM document_classifications dc
         INNER JOIN classification_nodes cn ON dc.node_id = cn.id
         INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
         WHERE dc.document_id = ?1
         ORDER BY cs.code, cn.notation",
    ) else {
        return Vec::new();
    };
    let rows = match stmt.query_map(rusqlite::params![doc_id], |row| {
        Ok(FullClassificationExport {
            scheme: row.get(0)?,
            notation: row.get(1)?,
            is_primary: row.get::<_, i64>(2)? != 0,
        })
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    rows.filter_map(|r| r.ok()).collect()
}

pub fn register_custom_format(config: ExportFormatConfig) {
    let mut guard = CUSTOM_FORMATS.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    map.insert(config.name.clone(), config);
}

pub fn register_custom_formats(configs: &[ExportFormatConfig]) {
    for c in configs {
        register_custom_format(c.clone());
    }
}

pub fn get_custom_format(name: &str) -> Option<ExportFormatConfig> {
    let guard = CUSTOM_FORMATS.lock().unwrap();
    guard.as_ref()?.get(name).cloned()
}

pub fn custom_format_names() -> Vec<String> {
    let guard = CUSTOM_FORMATS.lock().unwrap();
    guard
        .as_ref()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn export(documents: &[Document], format: ExportFormat, writer: &mut impl Write) -> Result<()> {
    match format {
        ExportFormat::BibliontologyRdf => {
            crate::citation::formats::bibliontology_rdf::export_bibliontology_rdf(documents, writer)
        }
        ExportFormat::Bibtex => crate::citation::bibtex::export_bibtex(documents, writer),
        ExportFormat::Bookmarks => {
            crate::citation::formats::bookmarks::export_bookmarks(documents, writer)
        }
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
        ExportFormat::Custom(name) => {
            let config = get_custom_format(&name)
                .ok_or_else(|| anyhow::anyhow!("unknown custom format: {}", name))?;
            export_custom(documents, &config, writer)
        }
    }
}

fn export_custom(
    documents: &[Document],
    config: &ExportFormatConfig,
    writer: &mut impl Write,
) -> Result<()> {
    for (i, doc) in documents.iter().enumerate() {
        if i > 0 {
            writeln!(writer)?;
        }
        let output = substitute_template(&config.template, doc);
        write!(writer, "{}", output)?;
    }
    Ok(())
}

fn substitute_template(template: &str, doc: &Document) -> String {
    template
        .replace("{title}", &doc.title)
        .replace("{authors}", doc.authors.as_deref().unwrap_or(""))
        .replace(
            "{year}",
            &doc.pub_year.map(|y| y.to_string()).unwrap_or_default(),
        )
        .replace("{doi}", doc.doi.as_deref().unwrap_or(""))
        .replace("{journal}", doc.journal.as_deref().unwrap_or(""))
        .replace("{abstract}", doc.abstract_text.as_deref().unwrap_or(""))
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    Custom(String),
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
            ExportFormat::Custom(_) => "txt",
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
            ExportFormat::Custom(name) => name.as_str(),
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

    pub fn as_str(&self) -> &str {
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
            ExportFormat::Custom(name) => name.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if let Some(fmt) = Self::all().iter().find(|fmt| fmt.as_str() == s) {
            return Some(fmt.clone());
        }
        if get_custom_format(s).is_some() {
            return Some(ExportFormat::Custom(s.to_string()));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ExportFormatConfig;
    use crate::db::documents::Document;
    use std::io::Cursor;

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

    #[test]
    fn test_custom_format_registered() {
        let config = ExportFormatConfig {
            name: "test_registered_fmt".to_string(),
            file_extension: "txt".to_string(),
            template: "{title} - {authors} ({year})".to_string(),
        };
        register_custom_format(config);

        assert!(get_custom_format("test_registered_fmt").is_some());
        assert!(ExportFormat::from_str("test_registered_fmt").is_some());
    }

    #[test]
    fn test_custom_format_generates() {
        let config = ExportFormatConfig {
            name: "test_gen_fmt".to_string(),
            file_extension: "txt".to_string(),
            template: "{title} | {authors} | {year} | {doi} | {journal} | {abstract}".to_string(),
        };
        register_custom_format(config);

        let doc = Document {
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            abstract_text: Some("A survey of deep learning methods.".to_string()),
            ..Default::default()
        };

        let mut buf = Vec::new();
        export(
            &[doc],
            ExportFormat::Custom("test_gen_fmt".to_string()),
            &mut Cursor::new(&mut buf),
        )
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("Deep Learning"), "missing title: {out}");
        assert!(out.contains("Smith, John"), "missing authors: {out}");
        assert!(out.contains("2023"), "missing year: {out}");
        assert!(out.contains("10.1234/test"), "missing doi: {out}");
        assert!(out.contains("Nature"), "missing journal: {out}");
        assert!(
            out.contains("A survey of deep learning methods."),
            "missing abstract: {out}"
        );
    }
}
