use anyhow::Result;
use rusqlite::Connection;

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS documents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    authors         TEXT,
    journal         TEXT,
    pub_year        INTEGER,
    doi             TEXT UNIQUE,
    arxiv_id        TEXT UNIQUE,
    abstract        TEXT,
    keywords        TEXT,
    file_path       TEXT,
    file_hash       TEXT,
    citation_key    TEXT UNIQUE,
    source          TEXT DEFAULT 'manual',
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_documents_doi ON documents(doi);
CREATE INDEX IF NOT EXISTS idx_documents_arxiv ON documents(arxiv_id);
CREATE INDEX IF NOT EXISTS idx_documents_year ON documents(pub_year);
CREATE INDEX IF NOT EXISTS idx_documents_citation_key ON documents(citation_key);

CREATE TABLE IF NOT EXISTS classification_schemes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT UNIQUE NOT NULL,
    name            TEXT NOT NULL,
    version         TEXT,
    enabled         INTEGER DEFAULT 1,
    is_primary      INTEGER DEFAULT 0,
    license         TEXT,
    source_url      TEXT,
    imported_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS classification_nodes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    scheme_id       INTEGER NOT NULL,
    notation        TEXT NOT NULL,
    pref_label      TEXT NOT NULL,
    alt_label       TEXT,
    scope_note      TEXT,
    parent_id       INTEGER,
    sort_order      INTEGER DEFAULT 0,
    FOREIGN KEY (scheme_id) REFERENCES classification_schemes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    UNIQUE(scheme_id, notation)
);

CREATE INDEX IF NOT EXISTS idx_nodes_scheme ON classification_nodes(scheme_id);
CREATE INDEX IF NOT EXISTS idx_nodes_parent ON classification_nodes(parent_id);
CREATE INDEX IF NOT EXISTS idx_nodes_notation ON classification_nodes(notation);

CREATE TABLE IF NOT EXISTS classification_labels (
    node_id         INTEGER NOT NULL,
    lang            TEXT NOT NULL,
    label           TEXT NOT NULL,
    source          TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    PRIMARY KEY (node_id, lang)
);

CREATE TABLE IF NOT EXISTS document_classifications (
    document_id     INTEGER NOT NULL,
    node_id         INTEGER NOT NULL,
    is_primary      INTEGER DEFAULT 0,
    confidence      REAL,
    assigned_by     TEXT,
    assigned_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_doc_class_doc ON document_classifications(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_class_node ON document_classifications(node_id);

CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_documents (
    project_id      INTEGER NOT NULL,
    document_id     INTEGER NOT NULL,
    added_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, document_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_proj_docs_project ON project_documents(project_id);
CREATE INDEX IF NOT EXISTS idx_proj_docs_doc ON project_documents(document_id);

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    authors,
    journal,
    abstract,
    keywords,
    content='documents',
    content_rowid='id',
    tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS trg_fts_insert AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, new.title, new.authors, new.journal, new.abstract, new.keywords);
END;

CREATE TRIGGER IF NOT EXISTS trg_fts_delete AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, old.title, old.authors, old.journal, old.abstract, old.keywords);
END;

CREATE TRIGGER IF NOT EXISTS trg_fts_update AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, old.title, old.authors, old.journal, old.abstract, old.keywords);
    INSERT INTO documents_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, new.title, new.authors, new.journal, new.abstract, new.keywords);
END;

CREATE VIRTUAL TABLE IF NOT EXISTS documents_bigram_fts USING fts5(
    title, authors, journal, abstract, keywords,
    content='', tokenize='unicode61'
);

CREATE TRIGGER IF NOT EXISTS trg_bigram_insert AFTER INSERT ON documents BEGIN
    INSERT INTO documents_bigram_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, bigrams_cjk(new.title), bigrams_cjk(new.authors),
            bigrams_cjk(new.journal), bigrams_cjk(new.abstract), bigrams_cjk(new.keywords));
END;

CREATE TRIGGER IF NOT EXISTS trg_bigram_delete AFTER DELETE ON documents BEGIN
    INSERT INTO documents_bigram_fts(documents_bigram_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, bigrams_cjk(old.title), bigrams_cjk(old.authors),
            bigrams_cjk(old.journal), bigrams_cjk(old.abstract), bigrams_cjk(old.keywords));
END;

CREATE TRIGGER IF NOT EXISTS trg_bigram_update AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_bigram_fts(documents_bigram_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, bigrams_cjk(old.title), bigrams_cjk(old.authors),
            bigrams_cjk(old.journal), bigrams_cjk(old.abstract), bigrams_cjk(old.keywords));
    INSERT INTO documents_bigram_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, bigrams_cjk(new.title), bigrams_cjk(new.authors),
            bigrams_cjk(new.journal), bigrams_cjk(new.abstract), bigrams_cjk(new.keywords));
END;

CREATE VIRTUAL TABLE IF NOT EXISTS documents_choseong_fts USING fts5(
    title, authors, journal, abstract, keywords,
    content='', tokenize='unicode61'
);

CREATE TRIGGER IF NOT EXISTS trg_choseong_insert AFTER INSERT ON documents BEGIN
    INSERT INTO documents_choseong_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, choseong_bigrams_cjk(new.title), choseong_bigrams_cjk(new.authors),
            choseong_bigrams_cjk(new.journal), choseong_bigrams_cjk(new.abstract), choseong_bigrams_cjk(new.keywords));
END;

CREATE TRIGGER IF NOT EXISTS trg_choseong_delete AFTER DELETE ON documents BEGIN
    INSERT INTO documents_choseong_fts(documents_choseong_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, choseong_bigrams_cjk(old.title), choseong_bigrams_cjk(old.authors),
            choseong_bigrams_cjk(old.journal), choseong_bigrams_cjk(old.abstract), choseong_bigrams_cjk(old.keywords));
END;

CREATE TRIGGER IF NOT EXISTS trg_choseong_update AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_choseong_fts(documents_choseong_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, choseong_bigrams_cjk(old.title), choseong_bigrams_cjk(old.authors),
            choseong_bigrams_cjk(old.journal), choseong_bigrams_cjk(old.abstract), choseong_bigrams_cjk(old.keywords));
    INSERT INTO documents_choseong_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, choseong_bigrams_cjk(new.title), choseong_bigrams_cjk(new.authors),
            choseong_bigrams_cjk(new.journal), choseong_bigrams_cjk(new.abstract), choseong_bigrams_cjk(new.keywords));
END;

CREATE TABLE IF NOT EXISTS api_cache (
    cache_key       TEXT PRIMARY KEY,
    source          TEXT NOT NULL,
    response_json   TEXT NOT NULL,
    fetched_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at      TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_expires ON api_cache(expires_at);

CREATE TABLE IF NOT EXISTS app_config (
    key             TEXT PRIMARY KEY,
    value           TEXT,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- User-assigned tags per document
CREATE TABLE IF NOT EXISTS tags (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id     INTEGER NOT NULL,
    tag             TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_tags_document ON tags(document_id);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

-- Citation relationships between documents
CREATE TABLE IF NOT EXISTS citation_relations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    citing_id       INTEGER NOT NULL,
    cited_id        INTEGER NOT NULL,
    FOREIGN KEY (citing_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (cited_id) REFERENCES documents(id) ON DELETE CASCADE,
    UNIQUE(citing_id, cited_id)
);

CREATE INDEX IF NOT EXISTS idx_citation_citing ON citation_relations(citing_id);
CREATE INDEX IF NOT EXISTS idx_citation_cited ON citation_relations(cited_id);
";

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}
