use crate::db::documents::Document;
use crate::export::fetch_user_export_data;
use anyhow::Result;
use rusqlite::Connection;
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

pub fn export_bibtex_with_user_data(
    conn: &Connection,
    documents: &[Document],
    writer: &mut impl Write,
) -> Result<()> {
    for doc in documents {
        let user_data = doc
            .id
            .and_then(|id| fetch_user_export_data(conn, id).ok())
            .unwrap_or_default();

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

        let mut keywords_parts: Vec<&str> = doc
            .keywords
            .as_deref()
            .filter(|s| !s.is_empty())
            .into_iter()
            .flat_map(|s| s.split(',').map(str::trim))
            .filter(|s| !s.is_empty())
            .collect();
        keywords_parts.extend(user_data.tags.iter().map(String::as_str));
        if !keywords_parts.is_empty() {
            writeln!(writer, "  keywords  = {{{}}},", keywords_parts.join(", "))?;
        }

        if let Some(ref path) = doc.file_path {
            writeln!(writer, "  file      = {{{}}},", path)?;
        }
        if !user_data.notes.is_empty() {
            writeln!(writer, "  note      = {{{}}},", user_data.notes.join(" "))?;
        }
        if !user_data.classifications.is_empty() {
            writeln!(
                writer,
                "  classification = {{{}}},",
                user_data.classifications.join(", ")
            )?;
        }
        writeln!(writer, "}}\n")?;
    }
    Ok(())
}

fn guess_entry_type(doc: &Document) -> &'static str {
    match doc.item_type.as_str() {
        "article" => "article",
        "book" => "book",
        "thesis" => "phdthesis",
        "conference" => "inproceedings",
        "dataset" => "misc",
        "webpage" => "misc",
        "patent" => "misc",
        _ => "misc",
    }
}
