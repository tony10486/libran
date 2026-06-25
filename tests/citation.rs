use libran::citation::{generate_citation_key, CitationKeyMode};
use libran::db::documents::Document;

#[test]
fn test_bibtex_export_format() {
    use libran::citation::bibtex;
    use std::io::Cursor;

    let doc = Document {
        id: Some(1),
        title: "Test Paper".to_string(),
        authors: Some("Smith, J. and Kim, D.".to_string()),
        journal: Some("Nature".to_string()),
        conference: None,
        pub_year: Some(2024),
        doi: Some("10.1000/test".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: Some("physics, quantum".to_string()),
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        citation_key: Some("Smith2024".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };

    let mut buf = Vec::new();
    bibtex::export_bibtex(&[doc], &mut Cursor::new(&mut buf)).unwrap();
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("@article {Smith2024"));
    assert!(output.contains("title     = {Test Paper}"));
    assert!(output.contains("author    = {Smith, J. and Kim, D.}"));
    assert!(output.contains("doi       = {10.1000/test}"));
}

#[test]
fn test_csl_json_export_format() {
    use libran::citation::csl_json;
    use std::io::Cursor;

    let doc = Document {
        id: Some(1),
        title: "Test Paper".to_string(),
        authors: Some("Smith, John".to_string()),
        journal: Some("Nature".to_string()),
        conference: None,
        pub_year: Some(2024),
        doi: Some("10.1000/test".to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        citation_key: Some("Smith2024".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };

    let mut buf = Vec::new();
    csl_json::export_csl_json(&[doc], &mut Cursor::new(&mut buf)).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed[0]["id"], "Smith2024");
    assert_eq!(parsed[0]["type"], "article-journal");
    assert_eq!(parsed[0]["title"], "Test Paper");
    assert_eq!(parsed[0]["doi"], "10.1000/test");
    assert_eq!(parsed[0]["author"][0]["family"], "Smith");
    assert_eq!(parsed[0]["author"][0]["given"], "John");
}

#[test]
fn test_collision_resolution_chain() {
    let doc = Document {
        id: None,
        title: "Paper".to_string(),
        authors: Some("Smith, J.".to_string()),
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        citation_key: None,
        source: None,
        rating: None,
        ..Default::default()
    };

    let existing: Vec<String> = vec!["Smith2024".into(), "Smith2024a".into(), "Smith2024b".into()];
    let key = generate_citation_key(&doc, &CitationKeyMode::AuthorYear, |k| {
        existing.contains(&k.to_string())
    });
    assert_eq!(key, "Smith2024c");
}

#[test]
fn test_custom_template_all_vars() {
    let doc = Document {
        id: None,
        title: "Network Analysis".to_string(),
        authors: Some("Lee, S.".to_string()),
        journal: None,
        conference: None,
        pub_year: Some(2024),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        citation_key: None,
        source: None,
        rating: None,
        ..Default::default()
    };

    let key = generate_citation_key(
        &doc,
        &CitationKeyMode::Custom("{author}_{year}_{titleword}".to_string()),
        |_| false,
    );
    assert_eq!(key, "Lee_2024_Network");
}
