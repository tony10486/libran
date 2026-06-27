use anyhow::Result;
use libran::api::openalex_forward::ForwardCitation;
use libran::app::forward_citations_handler::persist_forward_citations;
use libran::db;
use libran::db::documents::Document;
use rusqlite::Connection;

fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    db::init_database(&conn)?;
    Ok(conn)
}

fn make_cited_doc(conn: &Connection, doi: &str, cite_key: &str) -> Result<i64> {
    let doc = Document {
        id: None,
        title: "Cited Paper".to_string(),
        authors: Some("Author, A.".to_string()),
        doi: Some(doi.to_string()),
        citation_key: Some(cite_key.to_string()),
        ..Default::default()
    };
    db::documents::insert(conn, &doc)
}

#[test]
fn test_forward_citations_persisted() -> Result<()> {
    // Given: a cited document and two forward citations from OpenAlex
    let conn = setup_db()?;
    let cited_id = make_cited_doc(&conn, "10.1000/cited", "Cited2024")?;

    let citations = vec![
        ForwardCitation {
            title: "Citing Paper 1".to_string(),
            year: Some(2023),
            doi: Some("10.2000/citing1".to_string()),
            authors: vec!["Smith, J.".to_string(), "Lee, K.".to_string()],
        },
        ForwardCitation {
            title: "Citing Paper 2".to_string(),
            year: Some(2022),
            doi: Some("10.2000/citing2".to_string()),
            authors: vec!["Wang, Y.".to_string()],
        },
    ];

    // When: persist the forward citations
    persist_forward_citations(&conn, cited_id, &citations)?;

    // Then: both citing documents are persisted with title, year, DOI, authors
    let doc1 = db::documents::find_by_doi(&conn, "10.2000/citing1")?;
    assert!(
        doc1.is_some(),
        "citing paper 1 should be persisted as a document"
    );
    let doc1 = doc1.unwrap();
    assert_eq!(doc1.title, "Citing Paper 1");
    assert_eq!(doc1.pub_year, Some(2023));
    assert_eq!(doc1.authors.as_deref(), Some("Smith, J.; Lee, K."));

    let doc2 = db::documents::find_by_doi(&conn, "10.2000/citing2")?;
    assert!(
        doc2.is_some(),
        "citing paper 2 should be persisted as a document"
    );

    // And: citation_relations edges exist (citing -> cited)
    let citing_docs = db::documents::get_citing_docs(&conn, cited_id)?;
    assert_eq!(
        citing_docs.len(),
        2,
        "cited doc should have 2 forward citation edges"
    );

    Ok(())
}

#[test]
fn test_forward_citations_dedup() -> Result<()> {
    // Given: a cited document and two forward citations with the SAME DOI
    let conn = setup_db()?;
    let cited_id = make_cited_doc(&conn, "10.1000/cited2", "Cited2024b")?;

    let citations = vec![
        ForwardCitation {
            title: "Same Paper".to_string(),
            year: Some(2023),
            doi: Some("10.3000/dup".to_string()),
            authors: vec!["Author, X.".to_string()],
        },
        ForwardCitation {
            title: "Same Paper Again".to_string(),
            year: Some(2023),
            doi: Some("10.3000/dup".to_string()),
            authors: vec!["Author, X.".to_string()],
        },
    ];

    // When: persist the forward citations
    persist_forward_citations(&conn, cited_id, &citations)?;

    // Then: only ONE document with that DOI exists (no duplicate)
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE doi = ?1",
        rusqlite::params!["10.3000/dup"],
        |row| row.get(0),
    )?;
    assert_eq!(count, 1, "no duplicate documents for same DOI");

    // And: the citation_relation edge still exists (citing -> cited)
    let citing_docs = db::documents::get_citing_docs(&conn, cited_id)?;
    assert_eq!(citing_docs.len(), 1, "one edge to the deduped citing doc");

    Ok(())
}
