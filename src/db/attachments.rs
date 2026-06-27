use anyhow::Result;
use rusqlite::{Connection, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub id: Option<i64>,
    pub document_id: i64,
    pub file_path: String,
    pub file_hash: Option<String>,
    pub attachment_type: String,
    pub label: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: Option<String>,
}

/// Insert a single attachment row. Returns the new row id.
pub fn insert(conn: &Connection, attachment: &Attachment) -> Result<i64> {
    conn.execute(
        "INSERT INTO document_attachments (document_id, file_path, file_hash, attachment_type, label, mime_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            attachment.document_id,
            attachment.file_path,
            attachment.file_hash,
            attachment.attachment_type,
            attachment.label,
            attachment.mime_type,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List all attachments for a document, ordered by insertion id.
pub fn list_for_doc(conn: &Connection, doc_id: i64) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, file_path, file_hash, attachment_type, label, mime_type, created_at
         FROM document_attachments WHERE document_id = ?1 ORDER BY id",
    )?;
    let attachments = stmt
        .query_map(params![doc_id], |row| {
            Ok(Attachment {
                id: row.get(0)?,
                document_id: row.get(1)?,
                file_path: row.get(2)?,
                file_hash: row.get(3)?,
                attachment_type: row.get(4)?,
                label: row.get(5)?,
                mime_type: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(attachments)
}

/// Get a single attachment by its row id.
pub fn get_by_id(conn: &Connection, attachment_id: i64) -> Result<Option<Attachment>> {
    let result = conn.query_row(
        "SELECT id, document_id, file_path, file_hash, attachment_type, label, mime_type, created_at
         FROM document_attachments WHERE id = ?1",
        params![attachment_id],
        |row| {
            Ok(Attachment {
                id: row.get(0)?,
                document_id: row.get(1)?,
                file_path: row.get(2)?,
                file_hash: row.get(3)?,
                attachment_type: row.get(4)?,
                label: row.get(5)?,
                mime_type: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    );
    match result {
        Ok(a) => Ok(Some(a)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete a single attachment row by its id.
pub fn delete(conn: &Connection, attachment_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM document_attachments WHERE id = ?1",
        params![attachment_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::documents::Document;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        db::init_database(&conn)?;
        Ok(conn)
    }

    fn make_doc(title: &str) -> Document {
        Document {
            title: title.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_multiple_attachments() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Multi Attach"))?;

        let id1 = insert(
            &conn,
            &Attachment {
                id: None,
                document_id: doc_id,
                file_path: "/lib/att1.pdf".to_string(),
                file_hash: Some("abc123".to_string()),
                attachment_type: "supplementary".to_string(),
                label: Some("Supp A".to_string()),
                mime_type: Some("application/pdf".to_string()),
                created_at: None,
            },
        )?;

        let id2 = insert(
            &conn,
            &Attachment {
                id: None,
                document_id: doc_id,
                file_path: "/lib/att2.epub".to_string(),
                file_hash: Some("def456".to_string()),
                attachment_type: "supplementary".to_string(),
                label: Some("Supp B".to_string()),
                mime_type: Some("application/epub+zip".to_string()),
                created_at: None,
            },
        )?;

        let attachments = list_for_doc(&conn, doc_id)?;
        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].id, Some(id1));
        assert_eq!(attachments[1].id, Some(id2));
        assert_eq!(attachments[0].file_path, "/lib/att1.pdf");
        assert_eq!(attachments[1].file_path, "/lib/att2.epub");
        Ok(())
    }

    #[test]
    fn test_non_pdf_attachment() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("EPUB Doc"))?;

        let id = insert(
            &conn,
            &Attachment {
                id: None,
                document_id: doc_id,
                file_path: "/lib/book.epub".to_string(),
                file_hash: None,
                attachment_type: "supplementary".to_string(),
                label: Some("EPUB version".to_string()),
                mime_type: Some("application/epub+zip".to_string()),
                created_at: None,
            },
        )?;

        let retrieved = get_by_id(&conn, id)?;
        assert!(retrieved.is_some());
        let att = retrieved.unwrap();
        assert_eq!(att.attachment_type, "supplementary");
        assert_eq!(att.file_path, "/lib/book.epub");
        assert_eq!(att.mime_type.as_deref(), Some("application/epub+zip"));
        Ok(())
    }

    #[test]
    fn test_attachment_hash() -> Result<()> {
        let conn = setup()?;
        let doc_id = db::documents::insert(&conn, &make_doc("Hash Doc"))?;

        let tmp = PathBuf::from(std::env::temp_dir()).join("libran_test_att_hash.bin");
        fs::write(&tmp, b"hello world")?;

        let hash = crate::storage::library::compute_file_hash(&tmp)?;
        let _ = fs::remove_file(&tmp);

        let id = insert(
            &conn,
            &Attachment {
                id: None,
                document_id: doc_id,
                file_path: "/lib/data.csv".to_string(),
                file_hash: Some(hash.clone()),
                attachment_type: "dataset".to_string(),
                label: Some("Dataset".to_string()),
                mime_type: Some("text/csv".to_string()),
                created_at: None,
            },
        )?;

        let att = get_by_id(&conn, id)?.unwrap();
        assert_eq!(att.file_hash.as_deref(), Some(hash.as_str()));
        assert_eq!(att.file_hash.as_ref().unwrap().len(), 64);
        Ok(())
    }

    #[test]
    fn test_primary_attachment_backward_compat() -> Result<()> {
        let conn = setup()?;

        let doc = Document {
            title: "Primary PDF".to_string(),
            file_path: Some("/lib/primary.pdf".to_string()),
            file_hash: Some("sha256hex".to_string()),
            ..Default::default()
        };
        let doc_id = db::documents::insert(&conn, &doc)?;

        let retrieved = db::documents::get_by_id(&conn, doc_id)?.unwrap();
        assert_eq!(retrieved.file_path.as_deref(), Some("/lib/primary.pdf"));
        assert_eq!(retrieved.file_hash.as_deref(), Some("sha256hex"));

        let attachments = list_for_doc(&conn, doc_id)?;
        assert_eq!(attachments.len(), 0);

        insert(
            &conn,
            &Attachment {
                id: None,
                document_id: doc_id,
                file_path: "/lib/supplement.pdf".to_string(),
                file_hash: Some("supplement_hash".to_string()),
                attachment_type: "supplementary".to_string(),
                label: Some("Supplement".to_string()),
                mime_type: None,
                created_at: None,
            },
        )?;

        let retrieved = db::documents::get_by_id(&conn, doc_id)?.unwrap();
        assert_eq!(retrieved.file_path.as_deref(), Some("/lib/primary.pdf"));

        let attachments = list_for_doc(&conn, doc_id)?;
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].file_path, "/lib/supplement.pdf");
        Ok(())
    }
}
