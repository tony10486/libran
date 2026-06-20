use anyhow::Result;
use libran::db;
use libran::db::documents::Document;
use rusqlite::Connection;

fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    db::init_database(&conn)?;
    Ok(conn)
}

#[test]
fn test_schema_creation() -> Result<()> {
    let conn = setup_db()?;
    let tables: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
        )?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for expected in [
        "documents",
        "projects",
        "project_documents",
        "classification_schemes",
        "classification_nodes",
        "classification_labels",
        "document_classifications",
        "documents_fts",
        "documents_fts_data",
        "api_cache",
        "app_config",
        "tags",
        "citation_relations",
    ] {
        assert!(tables.contains(&expected.to_string()), "missing table: {}", expected);
    }
    Ok(())
}

#[test]
fn test_document_insert_and_retrieve() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "Test Paper on Networks".to_string(),
        authors: Some("Smith, J.".to_string()),
        journal: Some("Nature".to_string()),
        conference: None,
        pub_year: Some(2024),
        doi: Some("10.1000/test".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,

        citation_key: Some("Smith2024".to_string()),
        source: Some("pdf_extract".to_string()),
    };

    let id = db::documents::insert(&conn, &doc)?;
    assert!(id > 0);

    let retrieved = db::documents::get_by_id(&conn, id)?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.title, "Test Paper on Networks");
    assert_eq!(retrieved.doi, Some("10.1000/test".to_string()));
    Ok(())
}

#[test]
fn test_doi_uniqueness() -> Result<()> {
    let conn = setup_db()?;
    let doc1 = Document {
        id: None,
        title: "Paper 1".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: Some("10.1000/unique".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: Some("Key1".to_string()),
        source: None,
    };
    db::documents::insert(&conn, &doc1)?;

    let doc2 = Document {
        id: None,
        title: "Paper 2".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: Some("10.1000/unique".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: Some("Key2".to_string()),
        source: None,
    };
    let result = db::documents::insert(&conn, &doc2);
    assert!(result.is_err(), "duplicate DOI should fail");
    Ok(())
}

#[test]
fn test_project_document_mapping() -> Result<()> {
    let conn = setup_db()?;
    let project_id = db::projects::create_project(&conn, "ML Research", None)?;

    let doc = Document {
        id: None,
        title: "ML Paper".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: Some("10.2000/ml".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: Some("ML2024".to_string()),
        source: None,
    };
    let doc_id = db::documents::insert(&conn, &doc)?;

    db::projects::add_document(&conn, project_id, doc_id)?;

    let docs = db::projects::list_documents(&conn, project_id)?;
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0], doc_id);
    Ok(())
}

#[test]
fn test_fts_trigram_search() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식해석학의 기초".to_string(),
        authors: Some("김, 대영".to_string()),
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: Some("Kim2024".to_string()),
        source: None,
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "방정식")?;
    assert!(!results.is_empty(), "trigram search should match CJK substring");
    Ok(())
}

#[test]
fn test_citation_key_exists_check() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "Test".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        citation_key: Some("UniqueKey".to_string()),
        source: None,
    };
    db::documents::insert(&conn, &doc)?;

    assert!(db::documents::citation_key_exists(&conn, "UniqueKey")?);
    assert!(!db::documents::citation_key_exists(&conn, "Nonexistent")?);
    Ok(())
}

#[test]
fn test_file_hash_dedup() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "Paper".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: Some("10.3000/hash".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: Some("abc123".to_string()),
        citation_key: Some("Hash1".to_string()),
        source: None,
    };
    db::documents::insert(&conn, &doc)?;

    let found = db::documents::find_by_hash(&conn, "abc123")?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Paper");

    let not_found = db::documents::find_by_hash(&conn, "xyz789")?;
    assert!(not_found.is_none());
    Ok(())
}
