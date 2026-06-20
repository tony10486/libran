use crate::api::ApiMode;
use tokio::sync::mpsc;

use super::AppAction;

pub struct AppState {
    pub active_project_id: Option<i64>,
    pub document_search_term: String,
    pub is_processing: bool,
    pub status_text: String,
    pub api_mode: ApiMode,
    pub action_tx: mpsc::Sender<AppAction>,
    pub document_count: usize,
}

impl AppState {
    pub fn new(action_tx: mpsc::Sender<AppAction>) -> Self {
        AppState {
            active_project_id: None,
            document_search_term: String::new(),
            is_processing: false,
            status_text: "준비됨".to_string(),
            api_mode: ApiMode::default(),
            action_tx,
            document_count: 0,
        }
    }

    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    pub fn start_processing(&mut self, msg: &str) {
        self.is_processing = true;
        self.status_text = msg.to_string();
    }

    pub fn finish_processing(&mut self, msg: &str) {
        self.is_processing = false;
        self.status_text = msg.to_string();
    }
}
