use crate::db::documents::Document;
use crate::export::fetch_user_data;
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

        let doc_id = doc.id.unwrap_or(0);
        let user_data = fetch_user_data(conn, doc_id).unwrap_or_default();

        let mut keywords: Vec<String> = doc
            .keywords
            .as_ref()
            .map(|k| {
                k.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        for tag in &user_data.tags {
            if !keywords.contains(tag) {
                keywords.push(tag.clone());
            }
        }
        if !keywords.is_empty() {
            writeln!(writer, "  keywords  = {{{}}},", keywords.join(", "))?;
        }

        if let Some(ref notes) = user_data.notes {
            if !notes.is_empty() {
                writeln!(writer, "  note      = {{{}}}", notes)?;
            }
        }

        if let Some(ref path) = doc.file_path {
            writeln!(writer, "  file      = {{{}}},", path)?;
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
