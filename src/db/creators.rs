use anyhow::Result;
use rusqlite::{Connection, params};

use crate::db::documents::split_authors;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Creator {
    pub id: Option<i64>,
    pub document_id: i64,
    pub creator_type: String,
    pub family: Option<String>,
    pub given: Option<String>,
    pub suffix: Option<String>,
    pub particles: Option<String>,
    pub literal: Option<String>,
    pub locale: Option<String>,
    pub order_index: i64,
}

/// Insert a single creator row. Returns the new row id.
pub fn insert(conn: &Connection, creator: &Creator) -> Result<i64> {
    conn.execute(
        "INSERT INTO creators (document_id, creator_type, family, given, suffix, particles, literal, locale, order_index)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            creator.document_id,
            creator.creator_type,
            creator.family,
            creator.given,
            creator.suffix,
            creator.particles,
            creator.literal,
            creator.locale,
            creator.order_index,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List all creators for a document, ordered by `order_index`.
pub fn list_for_doc(conn: &Connection, doc_id: i64) -> Result<Vec<Creator>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, creator_type, family, given, suffix, particles, literal, locale, order_index
         FROM creators WHERE document_id = ?1 ORDER BY order_index",
    )?;
    let creators = stmt
        .query_map(params![doc_id], |row| {
            Ok(Creator {
                id: row.get(0)?,
                document_id: row.get(1)?,
                creator_type: row.get(2)?,
                family: row.get(3)?,
                given: row.get(4)?,
                suffix: row.get(5)?,
                particles: row.get(6)?,
                literal: row.get(7)?,
                locale: row.get(8)?,
                order_index: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(creators)
}

/// Delete all creator rows for a document.
pub fn delete_for_doc(conn: &Connection, doc_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM creators WHERE document_id = ?1",
        params![doc_id],
    )?;
    Ok(())
}

/// Detect the CJK locale of a name from its Unicode character ranges.
/// Returns "ko" for Hangul, "ja" for Kana, "zh" for Han-only.
/// Hangul and Kana take priority over Han because they are unambiguous.
fn detect_locale(name: &str) -> Option<&'static str> {
    let mut has_hangul = false;
    let mut has_kana = false;
    let mut has_han = false;
    for ch in name.chars() {
        let c = ch as u32;
        if (0xAC00..=0xD7AF).contains(&c) {
            has_hangul = true;
        } else if (0x3040..=0x309F).contains(&c) || (0x30A0..=0x30FF).contains(&c) {
            has_kana = true;
        } else if (0x4E00..=0x9FFF).contains(&c) || (0x3400..=0x4DBF).contains(&c) {
            has_han = true;
        }
    }
    if has_hangul {
        Some("ko")
    } else if has_kana {
        Some("ja")
    } else if has_han {
        Some("zh")
    } else {
        None
    }
}

/// Sync creator rows from the `authors` TEXT field.
/// Deletes existing creators for the doc, then inserts new rows by splitting
/// the authors string. CJK names without a comma go to `literal` (ambiguous
/// name order); CJK names with a comma split into family/given. Non-CJK names
/// split on comma or by last-word-is-family convention.
pub fn sync_from_authors(conn: &Connection, doc_id: i64, authors: Option<&str>) -> Result<()> {
    delete_for_doc(conn, doc_id)?;

    let authors = match authors {
        Some(a) if !a.trim().is_empty() => a,
        _ => return Ok(()),
    };

    for (idx, name) in split_authors(authors).iter().enumerate() {
        let locale = detect_locale(name);
        let (family, given, literal) = split_name(name, locale);

        insert(
            conn,
            &Creator {
                id: None,
                document_id: doc_id,
                creator_type: "author".to_string(),
                family,
                given,
                suffix: None,
                particles: None,
                literal,
                locale: locale.map(|l| l.to_string()),
                order_index: idx as i64,
            },
        )?;
    }

    Ok(())
}

/// Split a single author name into (family, given, literal) components.
/// CJK + no comma → literal (ambiguous name order).
/// CJK + comma → family/given split.
/// Non-CJK + comma → family/given split.
/// Non-CJK + no comma → last word is family, rest is given.
fn split_name(
    name: &str,
    locale: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    if locale.is_some() {
        if let Some(comma_pos) = name.find(',') {
            return (
                Some(name[..comma_pos].trim().to_string()),
                Some(name[comma_pos + 1..].trim().to_string()),
                None,
            );
        }
        return (None, None, Some(name.to_string()));
    }

    if let Some(comma_pos) = name.find(',') {
        return (
            Some(name[..comma_pos].trim().to_string()),
            Some(name[comma_pos + 1..].trim().to_string()),
            None,
        );
    }

    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => (None, None, None),
        1 => (Some(words[0].to_string()), None, None),
        _ => (
            Some(words.last().unwrap().to_string()),
            Some(words[..words.len() - 1].join(" ")),
            None,
        ),
    }
}

/// Backfill creator rows for all existing documents from their `authors` TEXT.
/// Called during migration M16. Idempotent: `sync_from_authors` deletes before
/// inserting, so re-running is safe.
pub fn backfill_from_documents(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, authors FROM documents WHERE authors IS NOT NULL AND trim(authors) <> ''",
    )?;
    let docs: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    for (doc_id, authors) in docs {
        sync_from_authors(conn, doc_id, Some(&authors))?;
    }
    Ok(())
}

/// Convert creator rows back to a semicolon-delimited authors string.
/// Uses `literal` if set (CJK ambiguous names), otherwise "family, given".
/// Round-trips with `split_authors`.
pub fn creators_to_authors_string(creators: &[Creator]) -> String {
    creators
        .iter()
        .map(|c| {
            if let Some(ref lit) = c.literal {
                return lit.clone();
            }
            match (&c.family, &c.given) {
                (Some(fam), Some(giv)) if !giv.is_empty() => format!("{fam}, {giv}"),
                (Some(fam), _) => fam.clone(),
                (None, Some(giv)) => giv.clone(),
                _ => String::new(),
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::documents::Document;
    use rusqlite::Connection;

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        db::init_database(&conn)?;
        Ok(conn)
    }

    fn make_doc(title: &str, authors: Option<&str>) -> Document {
        Document {
            title: title.to_string(),
            authors: authors.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_creator_insert_with_role() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Role Test", None))?;

        let id = insert(
            &conn,
            &Creator {
                id: None,
                document_id: doc_id,
                creator_type: "editor".to_string(),
                family: Some("Smith".to_string()),
                given: Some("John".to_string()),
                suffix: None,
                particles: None,
                literal: None,
                locale: None,
                order_index: 0,
            },
        )?;

        let creators = list_for_doc(&conn, doc_id)?;
        assert_eq!(creators.len(), 1);
        assert_eq!(creators[0].id, Some(id));
        assert_eq!(creators[0].creator_type, "editor");
        Ok(())
    }

    #[test]
    fn test_creator_order() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Order Test", None))?;

        for (idx, (fam, giv)) in [("Smith", "John"), ("Lee", "Jane"), ("Brown", "Bob")]
            .iter()
            .enumerate()
        {
            insert(
                &conn,
                &Creator {
                    id: None,
                    document_id: doc_id,
                    creator_type: "author".to_string(),
                    family: Some(fam.to_string()),
                    given: Some(giv.to_string()),
                    suffix: None,
                    particles: None,
                    literal: None,
                    locale: None,
                    order_index: idx as i64,
                },
            )?;
        }

        let creators = list_for_doc(&conn, doc_id)?;
        assert_eq!(creators.len(), 3);
        assert_eq!(creators[0].family.as_deref(), Some("Smith"));
        assert_eq!(creators[1].family.as_deref(), Some("Lee"));
        assert_eq!(creators[2].family.as_deref(), Some("Brown"));
        Ok(())
    }

    #[test]
    fn test_creator_cjk_locale() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("CJK Paper", Some("김철수")))?;

        let creators = list_for_doc(&conn, doc_id)?;
        assert_eq!(creators.len(), 1);
        assert_eq!(creators[0].locale.as_deref(), Some("ko"));
        assert_eq!(creators[0].literal.as_deref(), Some("김철수"));
        assert!(creators[0].family.is_none());
        assert!(creators[0].given.is_none());
        Ok(())
    }

    #[test]
    fn test_dual_write() -> Result<()> {
        let conn = setup()?;
        let doc = make_doc("Dual Write Test", Some("Smith, John; Lee, Jane"));
        let doc_id = db::documents::insert(&conn, &doc)?;

        let retrieved = db::documents::get_by_id(&conn, doc_id)?;
        assert_eq!(
            retrieved.unwrap().authors.as_deref(),
            Some("Smith, John; Lee, Jane")
        );

        let creators = list_for_doc(&conn, doc_id)?;
        assert_eq!(creators.len(), 2);
        assert_eq!(creators[0].family.as_deref(), Some("Smith"));
        assert_eq!(creators[0].given.as_deref(), Some("John"));
        assert_eq!(creators[1].family.as_deref(), Some("Lee"));
        assert_eq!(creators[1].given.as_deref(), Some("Jane"));
        Ok(())
    }

    #[test]
    fn test_backfill() -> Result<()> {
        let conn = setup()?;

        // Insert a doc via raw SQL to bypass dual-write (simulate pre-migration data)
        conn.execute(
            "INSERT INTO documents (title, authors, item_type) VALUES (?1, ?2, ?3)",
            params!["Backfill Test", "Smith, John; 김철수; Brown, Bob", "misc"],
        )?;
        let doc_id = conn.last_insert_rowid();

        // Ensure no creators exist for this doc
        delete_for_doc(&conn, doc_id)?;
        assert_eq!(list_for_doc(&conn, doc_id)?.len(), 0);

        // Run backfill
        backfill_from_documents(&conn)?;

        let creators = list_for_doc(&conn, doc_id)?;
        let split = split_authors("Smith, John; 김철수; Brown, Bob");
        assert_eq!(creators.len(), split.len());

        // First: non-CJK with comma
        assert_eq!(creators[0].family.as_deref(), Some("Smith"));
        assert_eq!(creators[0].given.as_deref(), Some("John"));
        assert!(creators[0].locale.is_none());

        // Second: CJK without comma → literal
        assert_eq!(creators[1].locale.as_deref(), Some("ko"));
        assert_eq!(creators[1].literal.as_deref(), Some("김철수"));

        // Third: non-CJK with comma
        assert_eq!(creators[2].family.as_deref(), Some("Brown"));
        assert_eq!(creators[2].given.as_deref(), Some("Bob"));
        Ok(())
    }

    #[test]
    fn test_creators_to_authors_string() -> Result<()> {
        let conn = setup()?;
        let authors = "Smith, John; 김철수; Brown, Bob";
        let doc_id = db::documents::insert(&conn, &make_doc("Round Trip", Some(authors)))?;

        let creators = list_for_doc(&conn, doc_id)?;
        let round_tripped = creators_to_authors_string(&creators);

        // Round-trip: re-splitting the output should match the original split
        assert_eq!(split_authors(&round_tripped), split_authors(authors));
        Ok(())
    }
}
