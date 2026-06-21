use std::collections::HashSet;

use tokio::sync::mpsc;

use crate::api::ApiMode;
use crate::config::AppConfig;
use crate::db::documents::Document;
use crate::db::facets::FacetCount;
use crate::db::projects::Project;
use crate::db::series::Series;
use crate::db::DbConn;
use crate::similarity::scoring::{DocumentScore, UdcTree};
use crate::similarity::SimilarityConfig;

use super::action::AppAction;
use super::graph_state::GraphState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelFocus {
    Left,
    Right,
    Detail,
    Graph,
}

pub struct AppState {
    pub db: DbConn,
    pub config: AppConfig,
    pub action_tx: mpsc::Sender<AppAction>,

    pub documents: Vec<Document>,
    pub projects: Vec<Project>,
    pub series: Vec<Series>,
    pub authors: Vec<(String, i64)>,
    pub facets: Vec<FacetCount>,

    pub active_panel: PanelFocus,
    pub list_cursor: usize,
    pub selected_doc_ids: HashSet<i64>,
    pub show_help: bool,

    pub search_mode: bool,
    pub search_input: String,

    pub add_file_mode: bool,
    pub add_file_input: String,

    pub show_detail: bool,
    pub detail_doc: Option<Document>,

    pub new_project_mode: bool,
    pub new_project_input: String,

    // ── Series bundling (optional grouping of same-series issues) ──
    pub series_grouping_enabled: bool,
    pub active_series_id: Option<i64>,
    pub new_series_mode: bool,
    pub new_series_input: String,

    pub edit_mode: bool,
    pub edit_field: usize,
    pub edit_input: String,
    pub edit_doc_id: Option<i64>,

    pub expanded_nodes: HashSet<String>,
    pub tree_cursor: usize,

    pub active_project_id: Option<i64>,
    pub active_author: Option<String>,
    pub is_processing: bool,
    pub status_text: String,
    pub api_mode: ApiMode,
    pub document_count: usize,
    pub dirty: bool,

    // ── Similarity sort ──
    pub similarity_ref_doc_id: Option<i64>,
    pub similarity_ref_title: String,
    pub similarity_scores: Vec<DocumentScore>,
    pub similarity_config: SimilarityConfig,
    pub udc_tree: UdcTree,

    // ── Citation graph ──
    pub graph_state: Option<GraphState>,
    pub citation_entry_mode: bool,
    pub citation_entry_cursor: usize,
    pub bibtex_import_mode: bool,
    pub bibtex_import_input: String,

    pub terminal_size: (u16, u16),

    // ── Note editing ──
    pub note_mode: bool,
    pub note_input: String,
    pub current_note: Option<String>,

    // ── Tags ──
    pub tag_mode: bool,
    pub tag_input: String,
    pub current_tags: Vec<String>,

    // ── Rating ──
    pub rating_mode: bool,

    pub pick_project_mode: bool,
    pub pick_project_input: String,
    pub pick_project_cursor: usize,

    pub authors_expanded: bool,
    pub author_search_mode: bool,
    pub author_search_input: String,
}

impl AppState {
    pub fn new(db: DbConn, config: AppConfig, action_tx: mpsc::Sender<AppAction>) -> Self {
        let api_mode = config.api_mode.clone();
        let similarity_config = SimilarityConfig::load();
        let udc_tree = load_udc_tree_from_db(&db);
        let series_grouping_enabled = load_series_grouping_enabled(&db);
        AppState {
            db,
            config,
            action_tx,
            documents: Vec::new(),
            projects: Vec::new(),
            series: Vec::new(),
            authors: Vec::new(),
            facets: Vec::new(),
            active_panel: PanelFocus::Left,
            list_cursor: 0,
            selected_doc_ids: HashSet::new(),
            show_help: false,
            search_mode: false,
            search_input: String::new(),
            add_file_mode: false,
            add_file_input: String::new(),
            show_detail: false,
            detail_doc: None,
            new_project_mode: false,
            new_project_input: String::new(),
            series_grouping_enabled,
            active_series_id: None,
            new_series_mode: false,
            new_series_input: String::new(),
            edit_mode: false,
            edit_field: 0,
            edit_input: String::new(),
            edit_doc_id: None,
            expanded_nodes: HashSet::new(),
            tree_cursor: 0,
            active_project_id: None,
            active_author: None,
            is_processing: false,
            status_text: "준비됨".to_string(),
            api_mode,
            document_count: 0,
            dirty: true,
            similarity_ref_doc_id: None,
            similarity_ref_title: String::new(),
            similarity_scores: Vec::new(),
            similarity_config,
            udc_tree,
            graph_state: None,
            citation_entry_mode: false,
            citation_entry_cursor: 0,
            bibtex_import_mode: false,
            bibtex_import_input: String::new(),
            terminal_size: (80, 24),
            note_mode: false,
            note_input: String::new(),
            current_note: None,
            tag_mode: false,
            tag_input: String::new(),
            current_tags: Vec::new(),
            rating_mode: false,
            pick_project_mode: false,
            pick_project_input: String::new(),
            pick_project_cursor: 0,
            authors_expanded: false,
            author_search_mode: false,
            author_search_input: String::new(),
        }
    }

    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
        self.dirty = true;
    }

    pub fn start_processing(&mut self, msg: &str) {
        self.is_processing = true;
        self.status_text = msg.to_string();
        self.dirty = true;
    }

    pub fn finish_processing(&mut self, msg: &str) {
        self.is_processing = false;
        self.status_text = msg.to_string();
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn reload_documents(&mut self) {
        if let Ok(conn) = self.db.lock() {
            if let Ok(docs) = crate::db::documents::list_all(&conn) {
                self.document_count = docs.len();
                if self.list_cursor >= docs.len() && !docs.is_empty() {
                    self.list_cursor = docs.len() - 1;
                }
                self.documents = docs;
            }
            if let Ok(facets) = crate::db::facets::count_by_classification(&conn, None, None) {
                self.facets = facets;
            }
            if let Ok(authors) = crate::db::documents::list_authors(&conn, 1) {
                self.authors = authors;
            }
        }
        self.dirty = true;
    }

    pub fn reload_projects(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(projects) = crate::db::projects::list_projects(&conn) {
                self.projects = projects;
            }
        self.dirty = true;
    }

    pub fn reload_series(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(series) = crate::db::series::list_series(&conn) {
                self.series = series;
            }
        self.dirty = true;
    }

    pub fn reload_authors(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(authors) = crate::db::documents::list_authors(&conn, 1) {
                self.authors = authors;
            }
        self.dirty = true;
    }

    pub fn load_detail(&mut self) {
        if let Some(doc) = self.documents.get(self.list_cursor) {
            self.detail_doc = Some(doc.clone());
            let doc_id = doc.id.unwrap_or(0);
            if let Ok(conn) = self.db.lock() {
                self.current_note = crate::db::notes::get(&conn, doc_id).ok().flatten();
                self.current_tags = crate::db::documents::get_tags(&conn, doc_id).unwrap_or_default();
            } else {
                self.current_note = None;
                self.current_tags = Vec::new();
            }
        }
    }

    pub fn reload_tags(&mut self) {
        if let Some(doc) = &self.detail_doc {
            let doc_id = doc.id.unwrap_or(0);
            if let Ok(conn) = self.db.lock() {
                self.current_tags = crate::db::documents::get_tags(&conn, doc_id).unwrap_or_default();
            }
        }
    }

    pub fn cycle_api_mode(&mut self) {
        self.api_mode = match self.api_mode {
            ApiMode::FullyOffline => ApiMode::IdentifierOnly,
            ApiMode::IdentifierOnly => ApiMode::AutoFallback,
            ApiMode::AutoFallback => ApiMode::FullyOffline,
            ApiMode::ManualSearch => ApiMode::AutoFallback,
        };
        self.config.api_mode = self.api_mode.clone();
        self.dirty = true;
    }

    pub fn init_classification(&mut self) {
        if let Ok(conn) = self.db.lock() {
            let _ = crate::classification::data_loader::load_all_schemes(&conn);
        }
    }

    pub fn reload_udc_tree(&mut self) {
        self.udc_tree = load_udc_tree_from_db(&self.db);
    }

    pub fn is_similarity_sorted(&self) -> bool {
        self.similarity_ref_doc_id.is_some()
    }
}

/// Load the UDC tree from the classification_nodes table in the database.
fn load_udc_tree_from_db(db: &DbConn) -> UdcTree {
    if let Ok(conn) = db.lock() {
        let mut stmt = match conn.prepare(
            "SELECT cn.notation, COALESCE(p.notation, '')
             FROM classification_nodes cn
             LEFT JOIN classification_nodes p ON cn.parent_id = p.id
             INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
             WHERE cs.code = 'udc'",
        ) {
            Ok(stmt) => stmt,
            Err(_) => return UdcTree::new(std::collections::HashMap::new()),
        };
        let mut parents = std::collections::HashMap::new();
        if let Ok(rows) = stmt.query_map([], |row| {
            let notation: String = row.get(0)?;
            let parent: String = row.get(1)?;
            Ok((notation, parent))
        }) {
            for row in rows.flatten() {
                parents.insert(row.0, row.1);
            }
        }
        UdcTree::new(parents)
    } else {
        UdcTree::new(std::collections::HashMap::new())
    }
}

fn load_series_grouping_enabled(db: &DbConn) -> bool {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'series_grouping_enabled'",
                [],
                |row| row.get(0),
            )
            .ok();
        return v.as_deref() == Some("true");
    }
    false
}
