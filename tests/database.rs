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
        "documents_bigram_fts",
        "documents_bigram_fts_data",
        "documents_choseong_fts",
        "documents_choseong_fts_data",
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
        citation_key: Some("Kim2024a".to_string()),
        source: None,
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
        citation_key: Some("Lee2023".to_string()),
        source: None,
    };
    let id1 = db::documents::insert(&conn, &doc1)?;
    let id2 = db::documents::insert(&conn, &doc2)?;

    let results = db::search::search_documents(&conn, "미분")?;
    assert!(results.contains(&id1), "2-char query '미분' should match '미분방정식 연구'");
    assert!(results.contains(&id2), "2-char query '미분' should match '편미분 방법론'");
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
        citation_key: Some("Kwak2024".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미분방")?;
    assert!(results.contains(&id), "3-char query '미분방' should match via trigram");
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
        citation_key: Some("Test1c".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미")?;
    assert!(results.contains(&id), "1-char query '미' should match via LIKE");
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
        citation_key: Some("Mixed1".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "미분")?;
    assert!(results.contains(&id), "2-char CJK '미분' should match mixed CJK+Latin title");
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
        citation_key: Some("FP1".to_string()),
        source: None,
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
        citation_key: Some("Eng1".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "differential")?;
    assert!(results.contains(&id), "English trigram search should still work");
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
        citation_key: Some("Eng2c".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "Qu")?;
    assert!(results.contains(&id), "2-char Latin query should match via LIKE");
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
        citation_key: Some("Sync1".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "초기")?;
    assert!(results.contains(&id), "insert → search should find doc");

    let mut updated = doc.clone();
    updated.id = Some(id);
    updated.title = "수정된 문서".to_string();
    db::documents::update(&conn, &updated)?;

    let old_results = db::search::search_documents(&conn, "초기")?;
    assert!(!old_results.contains(&id), "after update, old title term should miss");

    let new_results = db::search::search_documents(&conn, "수정")?;
    assert!(new_results.contains(&id), "after update, new title term should hit");

    db::documents::delete(&conn, id)?;
    let del_results = db::search::search_documents(&conn, "수정")?;
    assert!(!del_results.contains(&id), "after delete, search should miss");
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
        citation_key: Some("Bigram1".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "방정")?;
    assert!(results.contains(&id), "2-char CJK '방정' should match via bigram index");

    let results = db::search::search_documents(&conn, "분방")?;
    assert!(results.contains(&id), "2-char CJK '분방' should match via bigram index");
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
        citation_key: Some("BigramSync".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "초기")?;
    assert!(results.contains(&id), "insert → 2-char search should find doc");

    let mut updated = doc.clone();
    updated.id = Some(id);
    updated.title = "수정된 문서".to_string();
    db::documents::update(&conn, &updated)?;

    let old_results = db::search::search_documents(&conn, "초기")?;
    assert!(!old_results.contains(&id), "after update, old 2-char term should miss");

    let new_results = db::search::search_documents(&conn, "수정")?;
    assert!(new_results.contains(&id), "after update, new 2-char term should hit");

    db::documents::delete(&conn, id)?;
    let del_results = db::search::search_documents(&conn, "수정")?;
    assert!(!del_results.contains(&id), "after delete, 2-char search should miss");
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
        citation_key: Some("NfcTest".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let retrieved = db::documents::get_by_id(&conn, id)?.unwrap();
    assert_eq!(retrieved.title, "미분방정식", "NFD input should be stored as NFC");
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
        citation_key: Some("BigramFP".to_string()),
        source: None,
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
        citation_key: Some("JpBigram".to_string()),
        source: None,
    };
    let id_jp = db::documents::insert(&conn, &doc_jp)?;

    let results = db::search::search_documents(&conn, "微分")?;
    assert!(results.contains(&id_jp), "2-char Chinese '微分' should match via bigram");

    let results = db::search::search_documents(&conn, "方程")?;
    assert!(results.contains(&id_jp), "2-char '方程' should match via bigram");
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
        citation_key: Some("MigV3".to_string()),
        source: None,
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
        citation_key: Some("Cho1".to_string()),
        source: None,
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
        citation_key: Some("Cho2".to_string()),
        source: None,
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
        citation_key: Some("Cho3".to_string()),
        source: None,
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
        citation_key: Some("ChoFP".to_string()),
        source: None,
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
        citation_key: Some("ChoSync".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let results = db::search::search_documents(&conn, "ㅊㄱ")?;
    assert!(results.contains(&id), "insert → choseong search should find doc");

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
        citation_key: Some("ChoGap".to_string()),
        source: None,
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
        citation_key: Some("NoteTest".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let note = db::notes::get(&conn, id)?;
    assert!(note.is_none(), "new document should have no note");

    db::notes::set(&conn, id, "중요한 논문 — 참고용")?;
    let note = db::notes::get(&conn, id)?;
    assert_eq!(note.as_deref(), Some("중요한 논문 — 참고용"));

    db::notes::set(&conn, id, "수정된 노트")?;
    let note = db::notes::get(&conn, id)?;
    assert_eq!(note.as_deref(), Some("수정된 노트"), "set should overwrite");

    db::notes::delete(&conn, id)?;
    let note = db::notes::get(&conn, id)?;
    assert!(note.is_none(), "delete should remove note");

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
        citation_key: Some("CascadeDel".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;
    db::notes::set(&conn, id, "삭제될 노트")?;

    db::documents::delete(&conn, id)?;
    let note = db::notes::get(&conn, id)?;
    assert!(note.is_none(), "note should be cascade-deleted with document");

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
        citation_key: Some("MultiNote".to_string()),
        source: None,
    };
    let id = db::documents::insert(&conn, &doc)?;

    let content = "첫째 줄\n둘째 줄\n셋째 줄";
    db::notes::set(&conn, id, content)?;
    let note = db::notes::get(&conn, id)?;
    assert_eq!(note.as_deref(), Some(content));

    Ok(())
}
