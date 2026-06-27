use crate::db::documents::{Document, split_authors};
use anyhow::Result;
use std::io::Write;

pub fn export_bibliontology_rdf(documents: &[Document], writer: &mut impl Write) -> Result<()> {
    write_prefixes(writer)?;
    for doc in documents {
        write_document(writer, doc)?;
    }
    Ok(())
}

fn write_prefixes(writer: &mut impl Write) -> Result<()> {
    writeln!(
        writer,
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ."
    )?;
    writeln!(writer, "@prefix bibo: <http://purl.org/ontology/bibo/> .")?;
    writeln!(writer, "@prefix dcterms: <http://purl.org/dc/terms/> .")?;
    writeln!(writer, "@prefix foaf: <http://xmlns.com/foaf/0.1/> .")?;
    writeln!(writer, "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .")?;
    writeln!(writer)?;
    Ok(())
}

fn escape_turtle_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn doc_type(doc: &Document) -> &'static str {
    if doc.journal.is_some() {
        "bibo:AcademicArticle"
    } else if doc.conference.is_some() {
        "bibo:Article"
    } else {
        "bibo:Book"
    }
}

fn item_uri(doc: &Document) -> String {
    match &doc.doi {
        Some(doi) => format!("<info:doi/{}>", doi),
        None => format!("_:item_{}", doc.id.unwrap_or(0)),
    }
}

fn split_name(name: &str) -> (String, String) {
    match name.find(',') {
        Some(pos) => {
            let surname = name[..pos].trim().to_string();
            let given = name[pos + 1..].trim().to_string();
            (surname, given)
        }
        None => (name.trim().to_string(), String::new()),
    }
}

fn write_person_blank_node(writer: &mut impl Write, name: &str) -> Result<()> {
    let (surname, given) = split_name(name);
    writeln!(writer, "[\n        a foaf:Person ;")?;
    if !surname.is_empty() {
        writeln!(
            writer,
            "        foaf:surname \"{}\" ;",
            escape_turtle_literal(&surname)
        )?;
    }
    if !given.is_empty() {
        writeln!(
            writer,
            "        foaf:givenName \"{}\" ;",
            escape_turtle_literal(&given)
        )?;
    }
    write!(writer, "    ]")?;
    Ok(())
}

fn write_document(writer: &mut impl Write, doc: &Document) -> Result<()> {
    let uri = item_uri(doc);
    let dtype = doc_type(doc);

    writeln!(writer, "{} a {} ;", uri, dtype)?;
    writeln!(
        writer,
        "    dcterms:title \"{}\" ;",
        escape_turtle_literal(&doc.title)
    )?;

    let authors: Vec<String> = doc
        .authors
        .as_deref()
        .map(split_authors)
        .unwrap_or_default();

    if !authors.is_empty() {
        write!(writer, "    dcterms:creator ")?;
        for (i, author) in authors.iter().enumerate() {
            if i > 0 {
                write!(writer, ", ")?;
            }
            write_person_blank_node(writer, author)?;
        }
        writeln!(writer, " ;")?;
    }

    if let Some(doi) = &doc.doi {
        writeln!(writer, "    bibo:doi \"{}\" ;", escape_turtle_literal(doi))?;
    }

    if let Some(abs) = &doc.abstract_text {
        writeln!(
            writer,
            "    dcterms:abstract \"{}\" ;",
            escape_turtle_literal(abs)
        )?;
    }

    if let Some(keywords) = &doc.keywords {
        for kw in keywords.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            writeln!(
                writer,
                "    dcterms:subject \"{}\" ;",
                escape_turtle_literal(kw)
            )?;
        }
    }

    if let Some(journal) = &doc.journal {
        writeln!(
            writer,
            "    dcterms:isPartOf [\n        a bibo:Journal ;\n        dcterms:title \"{}\" ;\n    ] ;",
            escape_turtle_literal(journal)
        )?;
    }

    if let Some(year) = doc.pub_year {
        writeln!(writer, "    dcterms:issued \"{}\"^^xsd:gYear ;", year)?;
    }

    if !authors.is_empty() {
        writeln!(writer, "    bibo:authorList [\n        a rdf:Seq ;")?;
        for (i, author) in authors.iter().enumerate() {
            write!(writer, "        rdf:_{} ", i + 1)?;
            write_person_blank_node(writer, author)?;
            writeln!(writer, " ;")?;
        }
        writeln!(writer, "    ] ;")?;
    }

    writeln!(writer, "    .")?;
    writeln!(writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_stub_returns_ok() {
        let mut buf = Vec::new();
        let result = export_bibliontology_rdf(&[], &mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rdf_journal_article_with_doi() {
        let doc = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John; Lee, Jane".to_string()),
            journal: Some("Nature".to_string()),
            pub_year: Some(2023),
            doi: Some("10.1234/test".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bibliontology_rdf(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        eprintln!("--- RDF OUTPUT ---\n{out}\n--- END ---");
        assert!(
            out.contains("bibo:AcademicArticle"),
            "missing bibo:AcademicArticle: {out}"
        );
        assert!(
            out.contains("dcterms:title"),
            "missing dcterms:title: {out}"
        );
        assert!(out.contains("foaf:Person"), "missing foaf:Person: {out}");
        assert!(
            out.contains("foaf:surname \"Smith\""),
            "missing foaf:surname Smith: {out}"
        );
        assert!(out.contains("bibo:doi"), "missing bibo:doi: {out}");
        assert!(
            out.contains("dcterms:isPartOf"),
            "missing dcterms:isPartOf: {out}"
        );
    }

    #[test]
    fn test_rdf_book_without_journal() {
        let doc = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bibliontology_rdf(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("bibo:Book"), "missing bibo:Book: {out}");
        assert!(
            !out.contains("dcterms:isPartOf"),
            "should not have dcterms:isPartOf for book: {out}"
        );
    }

    #[test]
    fn test_rdf_keywords_produce_subject_triples() {
        let doc = Document {
            id: Some(1),
            title: "Deep Learning".to_string(),
            authors: Some("Smith, John".to_string()),
            keywords: Some("AI, ML".to_string()),
            ..Default::default()
        };
        let mut buf = Vec::new();
        export_bibliontology_rdf(&[doc], &mut Cursor::new(&mut buf)).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let count = out.matches("dcterms:subject").count();
        assert_eq!(
            count, 2,
            "expected 2 dcterms:subject lines, got {count}: {out}"
        );
    }
}
