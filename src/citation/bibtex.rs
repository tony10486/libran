use crate::db::documents::Document;
use anyhow::Result;
use std::io::Write;

pub fn export_bibtex(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    for doc in documents {
        let key = doc.citation_key.as_deref().unwrap_or("unknown");
        let entry_type = guess_entry_type(doc);
        writeln!(writer, "@{} {{{},", entry_type, key)?;
        if let Some(ref authors) = doc.authors {
            writeln!(writer, "  author    = {{{}}},", authors)?;
        }
        writeln!(writer, "  title     = {{{}}},", doc.title)?;
        if let Some(ref journal) = doc.journal {
            writeln!(writer, "  journal   = {{{}}},", journal)?;
        }
        if let Some(year) = doc.pub_year {
            writeln!(writer, "  year      = {{{}}},", year)?;
        }
        if let Some(ref doi) = doc.doi {
            writeln!(writer, "  doi       = {{{}}},", doi)?;
        }
        if let Some(ref arxiv) = doc.arxiv_id {
            writeln!(writer, "  eprint    = {{{}}},", arxiv)?;
            writeln!(writer, "  archivePrefix = {{arXiv}},")?;
        }
        if let Some(ref keywords) = doc.keywords {
            writeln!(writer, "  keywords  = {{{}}},", keywords)?;
        }
        if let Some(ref path) = doc.file_path {
            writeln!(writer, "  file      = {{{}}},", path)?;
        }
        writeln!(writer, "}}\n")?;
    }
    Ok(())
}

fn guess_entry_type(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "article"
    } else if doc.doi.as_ref().map(|d| d.contains("book")).unwrap_or(false) {
        "book"
    } else {
        "misc"
    }
}
