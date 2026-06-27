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
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
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
        "documents_bigram_fts",
        "documents_bigram_fts_data",
        "documents_choseong_fts",
        "documents_choseong_fts_data",
        "api_cache",
        "app_config",
        "tags",
        "citation_relations",
        "series",
        "document_series",
        "document_custom_fields",
    ] {
        assert!(
            tables.contains(&expected.to_string()),
            "missing table: {}",
            expected
        );
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,

        citation_key: Some("Smith2024".to_string()),
        source: Some("pdf_extract".to_string()),
        rating: None,
        ..Default::default()
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Key1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Key2".to_string()),
        source: None,
        rating: None,
        ..Default::default()
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("ML2024".to_string()),
        source: None,
        rating: None,
        ..Default::default()
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Kim2024".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "방정식")?;
    assert!(
        !results.is_empty(),
        "trigram search should match CJK substring"
    );
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("UniqueKey".to_string()),
        source: None,
        rating: None,
        ..Default::default()
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
        rating: None,
        ..Default::default()
    };
    db::documents::insert(&conn, &doc)?;

    let found = db::documents::find_by_hash(&conn, "abc123")?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Paper");

    let not_found = db::documents::find_by_hash(&conn, "xyz789")?;
    assert!(not_found.is_none());
    Ok(())
}

#[test]
fn test_korean_substring_2char() -> Result<()> {
    let conn = setup_db()?;
    let doc1 = Document {
        id: None,
        title: "미분방정식 연구".to_string(),
        authors: Some("김영수".to_string()),
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
        queue_position: None,
        citation_key: Some("Kim2024a".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let doc2 = Document {
        id: None,
        title: "편미분 방법론".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: Some(2023),
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Lee2023".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id1 = db::documents::insert(&conn, &doc1)?;
    let id2 = db::documents::insert(&conn, &doc2)?;

    let results = db::search::search_documents(&conn, "미분")?;
    assert!(
        results.contains(&id1),
        "2-char query '미분' should match '미분방정식 연구'"
    );
    assert!(
        results.contains(&id2),
        "2-char query '미분' should match '편미분 방법론'"
    );
    Ok(())
}

#[test]
fn test_korean_substring_3char_trigram() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식해석학의 기초".to_string(),
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Kwak2024".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미분방")?;
    assert!(
        results.contains(&id),
        "3-char query '미분방' should match via trigram"
    );
    Ok(())
}

#[test]
fn test_korean_1char_via_like() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Test1c".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미")?;
    assert!(
        results.contains(&id),
        "1-char query '미' should match via LIKE"
    );
    Ok(())
}

#[test]
fn test_mixed_cjk_latin_search() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "PDE 미분방정식 해석".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Mixed1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미분")?;
    assert!(
        results.contains(&id),
        "2-char CJK '미분' should match mixed CJK+Latin title"
    );
    Ok(())
}

#[test]
fn test_no_false_positive_korean() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("FP1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "분미")?;
    assert!(results.is_empty(), "'분미' should NOT match '미분방정식'");
    Ok(())
}

#[test]
fn test_english_regression_trigram() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "Partial Differential Equations in Physics".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Eng1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "differential")?;
    assert!(
        results.contains(&id),
        "English trigram search should still work"
    );
    Ok(())
}

#[test]
fn test_english_2char_like_fallback() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "Quantum Mechanics".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Eng2c".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "Qu")?;
    assert!(
        results.contains(&id),
        "2-char Latin query should match via LIKE"
    );
    Ok(())
}

#[test]
fn test_fts_trigger_sync() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "초기 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Sync1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "초기")?;
    assert!(results.contains(&id), "insert → search should find doc");

    let mut updated = doc.clone();
    updated.id = Some(id);
    updated.title = "수정된 문서".to_string();
    db::documents::update(&conn, &updated)?;

    let old_results = db::search::search_documents(&conn, "초기")?;
    assert!(
        !old_results.contains(&id),
        "after update, old title term should miss"
    );

    let new_results = db::search::search_documents(&conn, "수정")?;
    assert!(
        new_results.contains(&id),
        "after update, new title term should hit"
    );

    db::documents::delete(&conn, id)?;
    let del_results = db::search::search_documents(&conn, "수정")?;
    assert!(
        !del_results.contains(&id),
        "after delete, search should miss"
    );
    Ok(())
}

#[test]
fn test_bigram_2char_cjk_match() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식 연구".to_string(),
        authors: Some("김영수".to_string()),
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
        queue_position: None,
        citation_key: Some("Bigram1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "방정")?;
    assert!(
        results.contains(&id),
        "2-char CJK '방정' should match via bigram index"
    );

    let results = db::search::search_documents(&conn, "분방")?;
    assert!(
        results.contains(&id),
        "2-char CJK '분방' should match via bigram index"
    );
    Ok(())
}

#[test]
fn test_bigram_trigger_sync() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "초기 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("BigramSync".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "초기")?;
    assert!(
        results.contains(&id),
        "insert → 2-char search should find doc"
    );

    let mut updated = doc.clone();
    updated.id = Some(id);
    updated.title = "수정된 문서".to_string();
    db::documents::update(&conn, &updated)?;

    let old_results = db::search::search_documents(&conn, "초기")?;
    assert!(
        !old_results.contains(&id),
        "after update, old 2-char term should miss"
    );

    let new_results = db::search::search_documents(&conn, "수정")?;
    assert!(
        new_results.contains(&id),
        "after update, new 2-char term should hit"
    );

    db::documents::delete(&conn, id)?;
    let del_results = db::search::search_documents(&conn, "수정")?;
    assert!(
        !del_results.contains(&id),
        "after delete, 2-char search should miss"
    );
    Ok(())
}

#[test]
fn test_nfc_normalized_at_rest() -> Result<()> {
    use unicode_normalization::UnicodeNormalization;
    let conn = setup_db()?;

    let nfd_title: String = "미분방정식".nfd().collect();
    let doc = Document {
        id: None,
        title: nfd_title,
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("NfcTest".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.title, "미분방정식",
        "NFD input should be stored as NFC"
    );
    Ok(())
}

#[test]
fn test_bigram_no_false_positive() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("BigramFP".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "분미")?;
    assert!(results.is_empty(), "'분미' should NOT match '미분방정식'");
    Ok(())
}

#[test]
fn test_bigram_japanese_and_chinese() -> Result<()> {
    let conn = setup_db()?;
    let doc_jp = Document {
        id: None,
        title: "微分方程式の解法".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("JpBigram".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id_jp = db::documents::insert(&conn, &doc_jp)?;

    let results = db::search::search_documents(&conn, "微分")?;
    assert!(
        results.contains(&id_jp),
        "2-char Chinese '微分' should match via bigram"
    );

    let results = db::search::search_documents(&conn, "方程")?;
    assert!(
        results.contains(&id_jp),
        "2-char '方程' should match via bigram"
    );
    Ok(())
}

#[test]
fn test_migration_v3_populates_bigram_table() -> Result<()> {
    let conn = setup_db()?;

    let doc = Document {
        id: None,
        title: "미분방정식".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("MigV3".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    conn.execute(
        "INSERT INTO documents_bigram_fts(documents_bigram_fts) VALUES('delete-all')",
        [],
    )?;
    conn.execute(
        "UPDATE app_config SET value = '2' WHERE key = 'db_version'",
        [],
    )?;

    db::init_database(&conn)?;

    let results = db::search::search_documents(&conn, "미분")?;
    assert!(
        results.contains(&id),
        "2-char search should work after migration v3 repopulates bigram table"
    );
    Ok(())
}

#[test]
fn test_choseong_2char_match() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식 연구".to_string(),
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
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Cho1".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅁㅂ")?;
    assert!(
        results.contains(&id),
        "초성 'ㅁㅂ' should match '미분방정식 연구'"
    );
    Ok(())
}

#[test]
fn test_choseong_3char_match() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식해석".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Cho2".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅁㅂㅈ")?;
    assert!(
        results.contains(&id),
        "초성 'ㅁㅂㅈ' should match '미분방정식해석' via FTS5 AND of bigrams"
    );
    Ok(())
}

#[test]
fn test_choseong_author_search() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "논문 제목".to_string(),
        authors: Some("홍길동".to_string()),
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("Cho3".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅎㄱ")?;
    assert!(
        results.contains(&id),
        "초성 'ㅎㄱ' should match author '홍길동'"
    );
    Ok(())
}

#[test]
fn test_choseong_no_false_positive() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분방정식".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("ChoFP".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅁㅈ")?;
    assert!(
        results.is_empty(),
        "'ㅁㅈ' should NOT match '미분방정식' — ㅁ and ㅈ are not consecutive choseong"
    );
    Ok(())
}

#[test]
fn test_choseong_trigger_sync() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "초기 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("ChoSync".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅊㄱ")?;
    assert!(
        results.contains(&id),
        "insert → choseong search should find doc"
    );

    let mut updated = doc.clone();
    updated.id = Some(id);
    updated.title = "수정된 문서".to_string();
    db::documents::update(&conn, &updated)?;

    let old_results = db::search::search_documents(&conn, "ㅊㄱ")?;
    assert!(
        !old_results.contains(&id),
        "after update, old choseong 'ㅊㄱ' should miss"
    );

    let new_results = db::search::search_documents(&conn, "ㅅㅈ")?;
    assert!(
        new_results.contains(&id),
        "after update, new choseong 'ㅅㅈ' (수정) should hit"
    );

    db::documents::delete(&conn, id)?;
    let del_results = db::search::search_documents(&conn, "ㅅㅈ")?;
    assert!(
        !del_results.contains(&id),
        "after delete, choseong search should miss"
    );
    Ok(())
}

#[test]
fn test_choseong_does_not_cross_latin_gap() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "미분 AB 방정".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("ChoGap".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let _id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅂㅂ")?;
    assert!(
        results.is_empty(),
        "'ㅂㅂ' should NOT match '미분 AB 방정' — ㅂ(분) and ㅂ(방) are separated by Latin text"
    );

    let results2 = db::search::search_documents(&conn, "ㅁㅂ")?;
    assert!(
        !results2.is_empty(),
        "'ㅁㅂ' should match '미분 AB 방정' — 미분 are consecutive"
    );
    Ok(())
}

#[test]
fn test_document_notes_crud() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "노트 테스트용 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("NoteTest".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let notes = db::notes::list(&conn, id)?;
    assert!(notes.is_empty(), "new document should have no notes");

    let note_id = db::notes::create(&conn, id, "중요한 논문 — 참고용", "general")?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert_eq!(
        note.as_ref().map(|n| n.content.as_str()),
        Some("중요한 논문 — 참고용")
    );

    db::notes::update(&conn, note_id, "수정된 노트")?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert_eq!(
        note.as_ref().map(|n| n.content.as_str()),
        Some("수정된 노트"),
        "update should overwrite"
    );

    db::notes::delete_by_id(&conn, note_id)?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert!(note.is_none(), "delete_by_id should remove note");

    Ok(())
}

#[test]
fn test_document_notes_cascade_delete() -> Result<()> {
    let conn = setup_db()?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    let doc = Document {
        id: None,
        title: "삭제될 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("CascadeDel".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;
    db::notes::create(&conn, id, "삭제될 노트", "general")?;

    db::documents::delete(&conn, id)?;
    let notes = db::notes::list(&conn, id)?;
    assert!(
        notes.is_empty(),
        "notes should be cascade-deleted with document"
    );

    Ok(())
}

#[test]
fn test_document_notes_multiline() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "여러 줄 노트".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("MultiNote".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let content = "첫째 줄\n둘째 줄\n셋째 줄";
    let note_id = db::notes::create(&conn, id, content, "general")?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert_eq!(note.as_ref().map(|n| n.content.as_str()), Some(content));

    Ok(())
}

#[test]
fn test_multi_note() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "다중 노트 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("MultiNote2".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let id1 = db::notes::create(&conn, id, "첫 번째 노트", "general")?;
    let id2 = db::notes::create(&conn, id, "두 번째 노트", "idea")?;
    assert!(id1 != id2, "two notes should have distinct ids");

    let notes = db::notes::list(&conn, id)?;
    assert_eq!(notes.len(), 2, "both notes should exist for same document");
    assert!(
        notes
            .iter()
            .any(|n| n.id == Some(id1) && n.content == "첫 번째 노트")
    );
    assert!(
        notes
            .iter()
            .any(|n| n.id == Some(id2) && n.content == "두 번째 노트")
    );
    assert_eq!(notes[0].note_type, "idea");
    assert_eq!(notes[1].note_type, "general");

    Ok(())
}

#[test]
fn test_note_crud() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "CRUD 노트 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("CrudNote".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let note_id = db::notes::create(&conn, id, "원본 내용", "general")?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert_eq!(note.as_ref().map(|n| n.content.as_str()), Some("원본 내용"));

    db::notes::update(&conn, note_id, "수정된 내용")?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert_eq!(
        note.as_ref().map(|n| n.content.as_str()),
        Some("수정된 내용")
    );

    db::notes::delete_by_id(&conn, note_id)?;
    let note = db::notes::get_by_id(&conn, note_id)?;
    assert!(note.is_none(), "deleted note should not be found");

    Ok(())
}

#[test]
fn test_note_list_ordered() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        id: None,
        title: "정렬 노트 문서".to_string(),
        authors: None,
        journal: None,
        conference: None,
        pub_year: None,
        doi: None,
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some("OrderNote".to_string()),
        source: None,
        rating: None,
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let id1 = db::notes::create(&conn, id, "오래된 노트", "general")?;
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let id2 = db::notes::create(&conn, id, "중간 노트", "general")?;
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let id3 = db::notes::create(&conn, id, "최신 노트", "general")?;

    let notes = db::notes::list(&conn, id)?;
    assert_eq!(notes.len(), 3);
    assert_eq!(
        notes[0].id,
        Some(id3),
        "most recently created should be first"
    );
    assert_eq!(notes[1].id, Some(id2));
    assert_eq!(notes[2].id, Some(id1), "oldest should be last");

    std::thread::sleep(std::time::Duration::from_millis(1100));
    db::notes::update(&conn, id1, "오래된 노트 (수정됨)")?;
    let notes = db::notes::list(&conn, id)?;
    assert_eq!(notes[0].id, Some(id1), "updated note should move to front");

    Ok(())
}

fn make_doc(title: &str, journal: Option<&str>, doi: &str, cite_key: &str) -> Document {
    Document {
        id: None,
        title: title.to_string(),
        authors: Some("Author, A.".to_string()),
        journal: journal.map(|s| s.to_string()),
        conference: None,
        pub_year: Some(2024),
        doi: Some(doi.to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: None,
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        citation_key: Some(cite_key.to_string()),
        source: None,
        rating: None,
        ..Default::default()
    }
}

#[test]
fn test_series_crud() -> Result<()> {
    let conn = setup_db()?;

    let s1 = db::series::create_series(
        &conn,
        "Lecture Notes in Math",
        Some("Springer"),
        Some("0025-5858"),
    )?;
    let s2 = db::series::create_series(&conn, "Journal of Number Theory", None, None)?;
    assert!(s1 > 0 && s2 > s1);

    let list = db::series::list_series(&conn)?;
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "Journal of Number Theory");
    assert_eq!(list[1].name, "Lecture Notes in Math");
    assert_eq!(list[1].publisher.as_deref(), Some("Springer"));
    assert_eq!(list[1].issn.as_deref(), Some("0025-5858"));

    let found = db::series::get_by_name(&conn, "Lecture Notes in Math")?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, Some(s1));

    let none = db::series::get_by_name(&conn, "Nonexistent")?;
    assert!(none.is_none());

    db::series::delete_series(&conn, s1)?;
    let after = db::series::list_series(&conn)?;
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].name, "Journal of Number Theory");

    Ok(())
}

#[test]
fn test_series_document_mapping() -> Result<()> {
    let conn = setup_db()?;
    let sid = db::series::create_series(&conn, "Number Theory Series", None, None)?;

    let d1 = db::documents::insert(&conn, &make_doc("Vol 1", Some("NTS"), "10.1/nt1", "NT1"))?;
    let d2 = db::documents::insert(&conn, &make_doc("Vol 2", Some("NTS"), "10.1/nt2", "NT2"))?;
    let d3 = db::documents::insert(&conn, &make_doc("Vol 3", Some("NTS"), "10.1/nt3", "NT3"))?;

    db::series::add_document(&conn, sid, d1, Some("1"), None)?;
    db::series::add_document(&conn, sid, d2, Some("2"), None)?;
    db::series::add_document(&conn, sid, d3, Some("3"), None)?;

    assert_eq!(db::series::count_documents(&conn, sid)?, 3);

    let docs = db::series::list_documents(&conn, sid)?;
    assert_eq!(docs.len(), 3);

    let series_for_d1 = db::series::list_series_for_document(&conn, d1)?;
    assert_eq!(series_for_d1.len(), 1);
    assert_eq!(series_for_d1[0].name, "Number Theory Series");

    db::series::remove_document(&conn, sid, d2)?;
    assert_eq!(db::series::count_documents(&conn, sid)?, 2);

    db::series::delete_series(&conn, sid)?;
    let after = db::series::list_series_for_document(&conn, d1)?;
    assert_eq!(
        after.len(),
        0,
        "delete_series should cascade to document_series"
    );

    Ok(())
}

#[test]
fn test_auto_group_by_journal() -> Result<()> {
    let conn = setup_db()?;

    db::documents::insert(
        &conn,
        &make_doc("Paper A", Some("J. Number Theory"), "10.1/a", "A2024"),
    )?;
    db::documents::insert(
        &conn,
        &make_doc("Paper B", Some("J. Number Theory"), "10.1/b", "B2024"),
    )?;
    db::documents::insert(
        &conn,
        &make_doc("Paper C", Some("J. Number Theory"), "10.1/c", "C2024"),
    )?;
    db::documents::insert(
        &conn,
        &make_doc("Paper D", Some("Phys. Rev. Lett."), "10.1/d", "D2024"),
    )?;
    db::documents::insert(
        &conn,
        &make_doc("Paper E", Some("Phys. Rev. Lett."), "10.1/e", "E2024"),
    )?;
    db::documents::insert(
        &conn,
        &make_doc("Lonely Paper", Some("Solo Journal"), "10.1/f", "F2024"),
    )?;

    let proposals = db::series::propose_series_by_journal(&conn)?;
    assert_eq!(proposals.len(), 2, "two journals with 2+ docs each");
    assert_eq!(proposals[0].name, "J. Number Theory");
    assert_eq!(proposals[0].document_ids.len(), 3);
    assert_eq!(proposals[1].name, "Phys. Rev. Lett.");
    assert_eq!(proposals[1].document_ids.len(), 2);

    let ids = db::series::auto_group_by_journal(&conn)?;
    assert_eq!(ids.len(), 2);

    let series = db::series::list_series(&conn)?;
    assert_eq!(series.len(), 2);

    let nts = db::series::get_by_name(&conn, "J. Number Theory")?.unwrap();
    assert_eq!(db::series::count_documents(&conn, nts.id.unwrap())?, 3);

    let prl = db::series::get_by_name(&conn, "Phys. Rev. Lett.")?.unwrap();
    assert_eq!(db::series::count_documents(&conn, prl.id.unwrap())?, 2);

    let solo = db::series::get_by_name(&conn, "Solo Journal")?;
    assert!(
        solo.is_none(),
        "single-doc journal should not become a series"
    );

    Ok(())
}

#[test]
fn test_propose_series_skips_existing() -> Result<()> {
    let conn = setup_db()?;

    db::documents::insert(&conn, &make_doc("P1", Some("Math J"), "10.1/p1", "P1"))?;
    db::documents::insert(&conn, &make_doc("P2", Some("Math J"), "10.1/p2", "P2"))?;

    db::series::create_series(&conn, "Math J", None, None)?;

    let proposals = db::series::propose_series_by_journal(&conn)?;
    assert!(
        proposals.is_empty(),
        "should skip journals that already have a series"
    );

    let ids = db::series::auto_group_by_journal(&conn)?;
    assert_eq!(ids.len(), 1, "auto_group reuses existing series");
    let series = db::series::list_series(&conn)?;
    assert_eq!(series.len(), 1);

    Ok(())
}

// ── Fuzzy duplicate detection (#15) ──

fn make_dup_doc(title: &str, authors: Option<&str>, year: Option<i64>) -> Document {
    Document {
        id: None,
        title: title.to_string(),
        authors: authors.map(|s| s.to_string()),
        pub_year: year,
        ..Default::default()
    }
}

#[test]
fn test_fuzzy_dup_similar_title() -> Result<()> {
    let conn = setup_db()?;
    db::documents::insert(
        &conn,
        &make_dup_doc(
            "Deep Learning for Image Recognition",
            Some("Smith, John"),
            Some(2024),
        ),
    )?;

    let query = make_dup_doc(
        "Deep Learning for Image Recogniton",
        Some("Smith, John"),
        Some(2024),
    );
    let dups = db::documents::find_duplicates(&conn, &query)?;

    assert!(
        !dups.is_empty(),
        "90%-similar title should be detected as duplicate"
    );
    let (_, score) = dups[0];
    assert!(score >= 0.75, "score {} should be >= 0.75 threshold", score);

    Ok(())
}

#[test]
fn test_fuzzy_dup_below_threshold() -> Result<()> {
    let conn = setup_db()?;
    db::documents::insert(
        &conn,
        &make_dup_doc(
            "Quantum Computing Principles and Applications",
            Some("Einstein, Albert"),
            Some(2020),
        ),
    )?;

    let query = make_dup_doc(
        "Completely Unrelated Biology Topic",
        Some("Darwin, Charles"),
        Some(2023),
    );
    let dups = db::documents::find_duplicates(&conn, &query)?;

    assert!(
        dups.is_empty(),
        "dissimilar title/author/year should not be flagged"
    );

    Ok(())
}

#[test]
fn test_fuzzy_dup_different_year_same_title() -> Result<()> {
    let conn = setup_db()?;
    db::documents::insert(
        &conn,
        &make_dup_doc("Machine Learning Basics", Some("Turing, Alan"), Some(2020)),
    )?;

    let query = make_dup_doc("Machine Learning Basics", Some("Turing, Alan"), Some(2024));
    let dups = db::documents::find_duplicates(&conn, &query)?;

    assert!(
        !dups.is_empty(),
        "same title/author should still be detected even with year diff"
    );
    let (_, score) = dups[0];
    assert!(
        score < 1.0,
        "score {} should be < 1.0 to reflect year mismatch",
        score
    );
    assert!(score >= 0.75, "score {} should still be >= 0.75", score);

    Ok(())
}

#[test]
fn test_tag_color_set_get() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        title: "Color Tag Test".to_string(),
        ..Default::default()
    };
    let doc_id = db::documents::insert(&conn, &doc)?;
    db::documents::add_tag(&conn, doc_id, "important")?;
    db::documents::add_tag(&conn, doc_id, "review")?;

    let colors = db::documents::get_tags_with_color(&conn)?;
    let important_color = colors
        .iter()
        .find(|(t, _)| t == "important")
        .and_then(|(_, c)| c.as_ref());
    assert!(important_color.is_none(), "new tag should have no color");

    db::documents::set_tag_color(&conn, "important", Some("#ff0000"))?;
    let colors = db::documents::get_tags_with_color(&conn)?;
    let important_color = colors
        .iter()
        .find(|(t, _)| t == "important")
        .and_then(|(_, c)| c.as_ref());
    assert_eq!(
        important_color,
        Some(&"#ff0000".to_string()),
        "color should be set"
    );

    let review_color = colors
        .iter()
        .find(|(t, _)| t == "review")
        .and_then(|(_, c)| c.as_ref());
    assert!(review_color.is_none(), "untouched tag should have no color");

    db::documents::set_tag_color(&conn, "important", None)?;
    let colors = db::documents::get_tags_with_color(&conn)?;
    let important_color = colors
        .iter()
        .find(|(t, _)| t == "important")
        .and_then(|(_, c)| c.as_ref());
    assert!(important_color.is_none(), "color should be cleared");

    Ok(())
}

#[test]
fn test_favorite_filter() -> Result<()> {
    let conn = setup_db()?;

    let fav = Document {
        title: "Favorite Paper".to_string(),
        rating: Some(5),
        ..Default::default()
    };
    db::documents::insert(&conn, &fav)?;

    let mid = Document {
        title: "Mid Paper".to_string(),
        rating: Some(3),
        ..Default::default()
    };
    db::documents::insert(&conn, &mid)?;

    let none = Document {
        title: "No Rating Paper".to_string(),
        ..Default::default()
    };
    db::documents::insert(&conn, &none)?;

    let favorites = db::documents::list_favorites(&conn)?;
    assert_eq!(favorites.len(), 1, "only rating=5 docs should be favorites");
    assert_eq!(favorites[0].title, "Favorite Paper");

    Ok(())
}

// ── Reading queue / TBR (#26) ──

#[test]
fn test_queue_add_remove() -> Result<()> {
    let conn = setup_db()?;

    let d1 = db::documents::insert(&conn, &make_doc("Queue A", None, "10.1/qa", "QA"))?;
    let d2 = db::documents::insert(&conn, &make_doc("Queue B", None, "10.1/qb", "QB"))?;
    let d3 = db::documents::insert(&conn, &make_doc("Queue C", None, "10.1/qc", "QC"))?;

    let queue = db::documents::get_queue(&conn)?;
    assert!(queue.is_empty(), "queue should start empty");

    db::documents::add_to_queue(&conn, d1)?;
    db::documents::add_to_queue(&conn, d2)?;

    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 2, "queue should have 2 items");
    assert_eq!(queue[0].title, "Queue A", "first added should be first");
    assert_eq!(queue[1].title, "Queue B", "second added should be second");

    db::documents::remove_from_queue(&conn, d1)?;

    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 1, "queue should have 1 item after removal");
    assert_eq!(
        queue[0].title, "Queue B",
        "remaining item should be Queue B"
    );

    db::documents::remove_from_queue(&conn, d3)?;
    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 1, "removing non-queued doc should be no-op");

    db::documents::add_to_queue(&conn, d1)?;
    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 2, "queue should have 2 items after re-adding");
    assert_eq!(queue[0].title, "Queue B", "Queue B should still be first");
    assert_eq!(queue[1].title, "Queue A", "re-added Queue A should be last");

    Ok(())
}

#[test]
fn test_queue_ordered() -> Result<()> {
    let conn = setup_db()?;

    let d1 = db::documents::insert(&conn, &make_doc("First", None, "10.1/o1", "O1"))?;
    let d2 = db::documents::insert(&conn, &make_doc("Second", None, "10.1/o2", "O2"))?;
    let d3 = db::documents::insert(&conn, &make_doc("Third", None, "10.1/o3", "O3"))?;

    db::documents::add_to_queue(&conn, d1)?;
    db::documents::add_to_queue(&conn, d2)?;
    db::documents::add_to_queue(&conn, d3)?;

    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 3);
    assert_eq!(queue[0].title, "First");
    assert_eq!(queue[1].title, "Second");
    assert_eq!(queue[2].title, "Third");

    db::documents::reorder_queue(&conn, d3, 0)?;

    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 3);
    assert_eq!(queue[0].title, "Third", "Third should now be first");
    assert_eq!(queue[1].title, "First", "First should shift to second");
    assert_eq!(queue[2].title, "Second", "Second should shift to third");

    db::documents::remove_from_queue(&conn, d1)?;
    let queue = db::documents::get_queue(&conn)?;
    assert_eq!(queue.len(), 2);
    assert_eq!(queue[0].title, "Third");
    assert_eq!(queue[1].title, "Second");

    Ok(())
}

#[test]
fn test_reading_progress_update() -> Result<()> {
    let conn = setup_db()?;

    let doc = Document {
        title: "Progress Test".to_string(),
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;

    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.reading_progress,
        Some(0),
        "new doc should have 0 reading progress"
    );

    db::documents::update_reading_progress(&conn, id, 50)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.reading_progress,
        Some(50),
        "progress should be 50 after update"
    );

    db::documents::update_reading_progress(&conn, id, 100)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.reading_progress,
        Some(100),
        "progress should be 100 after update"
    );

    db::documents::update_reading_progress(&conn, id, 0)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.reading_progress,
        Some(0),
        "progress should be 0 after reset"
    );

    Ok(())
}

#[test]
fn test_item_type_default() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        title: "No Journal Doc".to_string(),
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "misc",
        "new doc without journal/isbn/conference should default to 'misc'"
    );
    Ok(())
}

#[test]
fn test_item_type_inferred() -> Result<()> {
    let conn = setup_db()?;

    let journal_doc = Document {
        title: "Journal Article".to_string(),
        journal: Some("Nature".to_string()),
        ..Default::default()
    };
    let journal_id = db::documents::insert(&conn, &journal_doc)?;

    let book_doc = Document {
        title: "Book Title".to_string(),
        isbn: Some("978-0-123456-78-9".to_string()),
        ..Default::default()
    };
    let book_id = db::documents::insert(&conn, &book_doc)?;

    let conf_doc = Document {
        title: "Conference Paper".to_string(),
        conference: Some("ICML 2023".to_string()),
        ..Default::default()
    };
    let conf_id = db::documents::insert(&conn, &conf_doc)?;

    // Run the same backfill SQL as migration M15
    conn.execute(
        "UPDATE documents SET item_type = 'article' WHERE journal IS NOT NULL AND item_type = 'misc'",
        [],
    )?;
    conn.execute(
        "UPDATE documents SET item_type = 'book' WHERE isbn IS NOT NULL AND item_type = 'misc'",
        [],
    )?;
    conn.execute(
        "UPDATE documents SET item_type = 'conference' WHERE conference IS NOT NULL AND item_type = 'misc'",
        [],
    )?;

    let retrieved = db::documents::get_by_id(&conn, journal_id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "article",
        "doc with journal should be inferred as 'article' after backfill"
    );

    let retrieved = db::documents::get_by_id(&conn, book_id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "book",
        "doc with isbn should be inferred as 'book' after backfill"
    );

    let retrieved = db::documents::get_by_id(&conn, conf_id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "conference",
        "doc with conference should be inferred as 'conference' after backfill"
    );

    Ok(())
}

#[test]
fn test_csl_json_uses_item_type() -> Result<()> {
    use libran::citation::csl_json;
    use std::io::Cursor;

    let conn = setup_db()?;
    let doc = Document {
        title: "Book Title".to_string(),
        item_type: "book".to_string(),
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;
    let doc = db::documents::get_by_id(&conn, id)?.unwrap();

    let mut buf = Vec::new();
    csl_json::export_csl_json(&[doc], &mut Cursor::new(&mut buf))?;
    let json: serde_json::Value = serde_json::from_slice(&buf)?;
    assert_eq!(
        json[0]["type"], "book",
        "CSL JSON type should be 'book' when item_type='book'"
    );
    Ok(())
}

#[test]
fn test_bibtex_uses_item_type() -> Result<()> {
    use libran::citation::bibtex;
    use std::io::Cursor;

    let conn = setup_db()?;
    let doc = Document {
        title: "Thesis Title".to_string(),
        item_type: "thesis".to_string(),
        citation_key: Some("smith2024thesis".to_string()),
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;
    let doc = db::documents::get_by_id(&conn, id)?.unwrap();

    let mut buf = Vec::new();
    bibtex::export_bibtex(&[doc], &mut Cursor::new(&mut buf))?;
    let output = String::from_utf8(buf)?;
    assert!(
        output.contains("@phdthesis {smith2024thesis"),
        "BibTeX should use @phdthesis when item_type='thesis': {output}"
    );
    Ok(())
}

#[test]
fn test_item_type_user_override() -> Result<()> {
    let conn = setup_db()?;
    let doc = Document {
        title: "Patent Doc".to_string(),
        item_type: "patent".to_string(),
        ..Default::default()
    };
    let id = db::documents::insert(&conn, &doc)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "patent",
        "user-set item_type='patent' should persist"
    );

    let mut doc = retrieved;
    doc.title = "Updated Patent Doc".to_string();
    db::documents::update(&conn, &doc)?;
    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(
        retrieved.item_type, "patent",
        "item_type should survive an update call"
    );
    assert_eq!(
        retrieved.title, "Updated Patent Doc",
        "title should be updated"
    );
    Ok(())
}
