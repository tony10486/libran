use anyhow::Result;
use libran::citation::bibtex::export_bibtex_with_user_data;
use libran::citation::csl_json::export_csl_json_with_user_data;
use libran::citation::formats::csv_export::export_csv_with_user_data;
use libran::citation::formats::ris::export_ris_with_user_data;
use libran::db;
use libran::db::documents::Document;
use libran::export::export_full_library_json;
use rusqlite::Connection;
use std::io::Cursor;

fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    db::init_database(&conn)?;
    Ok(conn)
}

fn insert_test_doc(conn: &Connection) -> Result<i64> {
    let doc = Document {
        title: "Test Paper".to_string(),
        authors: Some("Smith, John".to_string()),
        journal: Some("Nature".to_string()),
        pub_year: Some(2024),
        doi: Some("10.1234/test".to_string()),
        citation_key: Some("smith2024".to_string()),
        reading_status: Some("read".to_string()),
        ..Default::default()
    };
    Ok(db::documents::insert(conn, &doc)?)
}

/// Insert a UDC scheme, a node, and link it to a document.
fn add_udc_classification(
    conn: &Connection,
    doc_id: i64,
    notation: &str,
    label: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO classification_schemes (code, name, version, enabled, is_primary)
         VALUES ('udc', 'UDC', '1', 1, 1)",
        [],
    )?;
    let scheme_id: i64 = conn.query_row(
        "SELECT id FROM classification_schemes WHERE code = 'udc'",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO classification_nodes (scheme_id, notation, pref_label, sort_order)
         VALUES (?1, ?2, ?3, 0)",
        rusqlite::params![scheme_id, notation, label],
    )?;
    let node_id: i64 = conn.query_row(
        "SELECT id FROM classification_nodes WHERE scheme_id = ?1 AND notation = ?2",
        rusqlite::params![scheme_id, notation],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO document_classifications (document_id, node_id, is_primary, assigned_by)
         VALUES (?1, ?2, 1, 'manual')",
        rusqlite::params![doc_id, node_id],
    )?;
    Ok(())
}

#[test]
fn test_csl_json_includes_tags() -> Result<()> {
    // Given a document with tags "AI" and "ML"
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    db::documents::add_tag(&conn, doc_id, "AI")?;
    db::documents::add_tag(&conn, doc_id, "ML")?;

    let docs = db::documents::list_all(&conn)?;

    // When exporting to CSL JSON with user data
    let mut buf = Vec::new();
    export_csl_json_with_user_data(&conn, &docs, &mut Cursor::new(&mut buf))?;
    let json = String::from_utf8(buf)?;

    // Then the tags appear in the keyword field
    assert!(
        json.contains("AI"),
        "tag 'AI' should be in CSL JSON: {json}"
    );
    assert!(
        json.contains("ML"),
        "tag 'ML' should be in CSL JSON: {json}"
    );
    Ok(())
}

#[test]
fn test_csv_includes_notes() -> Result<()> {
    // Given a document with a note
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    db::notes::set(&conn, doc_id, "This is a test note")?;

    let docs = db::documents::list_all(&conn)?;

    // When exporting to CSV with user data
    let mut buf = Vec::new();
    export_csv_with_user_data(&conn, &docs, &mut Cursor::new(&mut buf))?;
    let csv = String::from_utf8(buf)?;

    // Then the CSV has a notes column and the note content
    assert!(
        csv.contains("notes"),
        "CSV header should have 'notes' column: {csv}"
    );
    assert!(
        csv.contains("This is a test note"),
        "CSV should contain note content: {csv}"
    );
    Ok(())
}

#[test]
fn test_full_library_export() -> Result<()> {
    // Given a document with tags, notes, classifications, and a project
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    db::documents::add_tag(&conn, doc_id, "physics")?;
    db::notes::set(&conn, doc_id, "Important note")?;
    add_udc_classification(&conn, doc_id, "53", "Physics")?;

    let proj_id = db::projects::create_project(&conn, "Research", None)?;
    db::projects::add_document(&conn, proj_id, doc_id)?;

    // When exporting the full library as JSON
    let json = export_full_library_json(&conn)?;

    // Then all user data is included
    assert!(
        json.contains("physics"),
        "tags should be in full export: {json}"
    );
    assert!(
        json.contains("Important note"),
        "notes should be in full export: {json}"
    );
    assert!(
        json.contains("\"53\""),
        "classification notation should be in full export: {json}"
    );
    assert!(
        json.contains("Research"),
        "project name should be in full export: {json}"
    );
    Ok(())
}

#[test]
fn test_export_includes_classifications() -> Result<()> {
    // Given a document with a UDC classification
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    add_udc_classification(&conn, doc_id, "510", "Mathematics")?;

    let docs = db::documents::list_all(&conn)?;

    // When exporting to CSL JSON with user data
    let mut buf = Vec::new();
    export_csl_json_with_user_data(&conn, &docs, &mut Cursor::new(&mut buf))?;
    let json = String::from_utf8(buf)?;

    // Then the classification notation and label appear in the output
    assert!(
        json.contains("510"),
        "classification notation should be in CSL JSON: {json}"
    );
    assert!(
        json.contains("Mathematics"),
        "classification label should be in CSL JSON: {json}"
    );
    Ok(())
}

#[test]
fn test_bibtex_includes_notes_and_tags() -> Result<()> {
    // Given a document with tags and a note
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    db::documents::add_tag(&conn, doc_id, "quantum")?;
    db::notes::set(&conn, doc_id, "Read this carefully")?;

    let docs = db::documents::list_all(&conn)?;

    // When exporting to BibTeX with user data
    let mut buf = Vec::new();
    export_bibtex_with_user_data(&conn, &docs, &mut Cursor::new(&mut buf))?;
    let bib = String::from_utf8(buf)?;

    // Then the note and tag appear in the BibTeX output
    assert!(
        bib.contains("quantum"),
        "tag should be in BibTeX keywords: {bib}"
    );
    assert!(
        bib.contains("Read this carefully"),
        "note should be in BibTeX note field: {bib}"
    );
    Ok(())
}

#[test]
fn test_ris_includes_notes_and_tags() -> Result<()> {
    // Given a document with tags and a note
    let conn = setup_db()?;
    let doc_id = insert_test_doc(&conn)?;
    db::documents::add_tag(&conn, doc_id, "gravity")?;
    db::notes::set(&conn, doc_id, "Important finding")?;

    let docs = db::documents::list_all(&conn)?;

    // When exporting to RIS with user data
    let mut buf = Vec::new();
    export_ris_with_user_data(&conn, &docs, &mut Cursor::new(&mut buf))?;
    let ris = String::from_utf8(buf)?;

    // Then the tag appears as KW and the note appears as N1
    assert!(
        ris.contains("KW  - gravity"),
        "tag should be in RIS KW: {ris}"
    );
    assert!(
        ris.contains("N1  - Important finding"),
        "note should be in RIS N1: {ris}"
    );
    Ok(())
}
