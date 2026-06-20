use std::collections::HashSet;

use tokio::sync::mpsc;

use crate::api::ApiMode;
use crate::config::AppConfig;
use crate::db::documents::Document;
use crate::db::facets::FacetCount;
use crate::db::projects::Project;
use crate::db::DbConn;

use super::AppAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelFocus {
    Left,
    Right,
    Detail,
}

pub struct AppState {
    pub db: DbConn,
    pub config: AppConfig,
    pub action_tx: mpsc::Sender<AppAction>,

    pub documents: Vec<Document>,
    pub projects: Vec<Project>,
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

    pub edit_mode: bool,
    pub edit_field: usize,
    pub edit_input: String,
    pub edit_doc_id: Option<i64>,

    pub expanded_nodes: HashSet<String>,
    pub tree_cursor: usize,

    pub active_project_id: Option<i64>,
    pub is_processing: bool,
    pub status_text: String,
    pub api_mode: ApiMode,
    pub document_count: usize,
    pub dirty: bool,
}

impl AppState {
    pub fn new(db: DbConn, config: AppConfig, action_tx: mpsc::Sender<AppAction>) -> Self {
        let api_mode = config.api_mode.clone();
        AppState {
            db,
            config,
            action_tx,
            documents: Vec::new(),
            projects: Vec::new(),
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
            edit_mode: false,
            edit_field: 0,
            edit_input: String::new(),
            edit_doc_id: None,
            expanded_nodes: HashSet::new(),
            tree_cursor: 0,
            active_project_id: None,
            is_processing: false,
            status_text: "준비됨".to_string(),
            api_mode,
            document_count: 0,
            dirty: true,
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

    pub fn load_detail(&mut self) {
        if let Some(doc) = self.documents.get(self.list_cursor) {
            self.detail_doc = Some(doc.clone());
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
}
