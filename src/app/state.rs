use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc;

use crate::api::ApiMode;
use crate::api::metrics::{AuthorMetrics, MetricsBackend};
use crate::config::AppConfig;
use crate::db::DbConn;
use crate::db::documents::Document;
use crate::db::facets::FacetCount;
use crate::db::projects::Project;
use crate::db::series::Series;
use crate::similarity::SimilarityConfig;
use crate::similarity::scoring::{DocumentScore, UdcTree};
use crate::ui::settings_panel::SettingsPanelState;

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
    pub help_page: usize,

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
    pub active_udc_notation: Option<String>,
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
    pub left_panel_width: u16,
    pub glyph_set: String,

    // ── Note editing ──
    pub note_mode: bool,
    pub note_input: String,
    pub current_notes: Vec<crate::db::notes::Note>,

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

    // ── Delete confirmation dialog ──
    pub confirm_delete_mode: bool,
    pub delete_confirm_doc_id: Option<i64>,
    pub delete_confirm_title: String,
    pub skip_delete_confirm: bool,

    // ── Sioyek install dialog ──
    pub show_sioyek_install_dialog: bool,
    pub sioyek_install_doc_id: Option<i64>,

    // ── Okular install dialog ──
    pub show_okular_install_dialog: bool,
    pub okular_install_doc_id: Option<i64>,

    // ── Author metrics (h-index, i10-index) ──
    pub metrics_backend: MetricsBackend,
    pub openalex_api_key: Option<String>,
    pub author_metrics: std::collections::HashMap<String, AuthorMetrics>,
    pub show_metrics_overlay: bool,
    pub metrics_overlay_name: String,
    pub api_key_input_mode: bool,
    pub api_key_input: String,
    pub auto_fetch_metrics: bool,
    pub metrics_refresh_interval_days: u32,

    // ── Custom metadata fields ──
    pub custom_fields: Vec<(i64, String, String)>,
    pub custom_field_mode: bool,
    pub custom_field_key: String,
    pub custom_field_value: String,
    pub custom_field_editing_key: bool,

    pub show_export_dialog: bool,
    pub export_dialog_state: crate::export::export_dialog_state::ExportDialogState,

    // ── Saved searches (G) ──
    pub saved_searches: Vec<crate::db::saved_searches::SavedSearch>,
    pub save_search_mode: bool,
    pub save_search_input: String,

    // ── Statistics dashboard (I) ──
    pub show_stats: bool,
    pub library_stats: Option<crate::db::stats::LibraryStats>,

    // ── PDF bookmarks (J) ──
    pub current_bookmarks: Vec<(String, i64)>,

    // ── Additional attachments (non-primary files) ──
    pub current_attachments: Vec<crate::db::attachments::Attachment>,

    // ── Bulk DOI import (C) ──
    pub bulk_import_mode: bool,
    pub bulk_import_input: String,

    // ── File import (F) ──
    pub file_import_mode: bool,
    pub file_import_input: String,

    // ── Author merge (E) ──
    pub author_merge_mode: bool,
    pub author_merge_phase: u8,
    pub author_merge_source: String,
    pub author_merge_input: String,

    // ── Settings panel (,) ──
    pub settings_panel_mode: bool,
    pub settings_panel: Option<SettingsPanelState>,

    // ── Command mode (:backup / :restore) ──
    pub command_mode: bool,
    pub command_input: String,

    // ── Favorite filter (*) ──
    pub favorite_filter: bool,

    // ── Reading queue / TBR (Y) ──
    pub queue_view: bool,
    pub queue: Vec<Document>,

    // ── Tag colors (tag name -> hex color) ──
    pub tag_colors: HashMap<String, String>,

    // ── Widget panel (w) ──
    pub show_widget_panel: bool,
    pub widget_registry: crate::widget::WidgetRegistry,
    pub widget_sandbox: crate::widget::sandbox::Sandbox,
}

impl AppState {
    pub fn new(db: DbConn, config: AppConfig, action_tx: mpsc::Sender<AppAction>) -> Self {
        let api_mode = config.api_mode.clone();
        let glyph_set = config.glyph_set.clone();
        let similarity_config = SimilarityConfig::load();
        let udc_tree = load_udc_tree_from_db(&db);
        let series_grouping_enabled = load_series_grouping_enabled(&db);
        let skip_delete_confirm = load_skip_delete_confirm(&db);
        let left_panel_width = load_left_panel_width(&db);
        let metrics_backend = load_metrics_backend(&db);
        let openalex_api_key = load_openalex_api_key(&db);
        let auto_fetch_metrics = load_auto_fetch_metrics(&db);
        let metrics_refresh_interval_days = load_metrics_refresh_interval_days(&db);
        let export_dialog_state = load_export_dialog_state(&db);

        // ── Widget 초기화 ──
        // 코어에는 위젯이 내장되어 있지 않습니다.
        // 모든 위젯은 ~/.libran/widgets/<name>/widget.toml 플러그인으로 로드됩니다.
        let widget_sandbox = crate::widget::sandbox::default_sandbox();
        let mut widget_registry = crate::widget::WidgetRegistry::new();
        // 플러그인 위젯 자동 탐색
        crate::widget::discovery::discover_plugin_widgets(&mut widget_registry, &widget_sandbox);
        // 예시 위젯 파일 생성 (최초 실행 시)
        crate::widget::discovery::write_example_widgets();

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
            help_page: 0,
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
            active_udc_notation: None,
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
            left_panel_width,
            glyph_set,
            note_mode: false,
            note_input: String::new(),
            current_notes: Vec::new(),
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
            confirm_delete_mode: false,
            delete_confirm_doc_id: None,
            delete_confirm_title: String::new(),
            skip_delete_confirm,
            show_sioyek_install_dialog: false,
            sioyek_install_doc_id: None,
            show_okular_install_dialog: false,
            okular_install_doc_id: None,
            metrics_backend,
            openalex_api_key,
            author_metrics: HashMap::new(),
            show_metrics_overlay: false,
            metrics_overlay_name: String::new(),
            api_key_input_mode: false,
            api_key_input: String::new(),
            auto_fetch_metrics,
            metrics_refresh_interval_days,
            custom_fields: Vec::new(),
            custom_field_mode: false,
            custom_field_key: String::new(),
            custom_field_value: String::new(),
            custom_field_editing_key: false,
            show_export_dialog: false,
            export_dialog_state,
            saved_searches: Vec::new(),
            save_search_mode: false,
            save_search_input: String::new(),
            show_stats: false,
            library_stats: None,
            current_bookmarks: Vec::new(),
            current_attachments: Vec::new(),
            bulk_import_mode: false,
            bulk_import_input: String::new(),
            file_import_mode: false,
            file_import_input: String::new(),
            author_merge_mode: false,
            author_merge_phase: 0,
            author_merge_source: String::new(),
            author_merge_input: String::new(),
            settings_panel_mode: false,
            settings_panel: None,
            command_mode: false,
            command_input: String::new(),
            favorite_filter: false,
            queue_view: false,
            queue: Vec::new(),
            tag_colors: HashMap::new(),
            show_widget_panel: false,
            widget_registry,
            widget_sandbox,
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
            let docs_result = if self.favorite_filter {
                crate::db::documents::list_favorites(&conn)
            } else {
                crate::db::documents::list_all(&conn)
            };
            if let Ok(docs) = docs_result {
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
            && let Ok(projects) = crate::db::projects::list_projects(&conn)
        {
            self.projects = projects;
        }
        self.dirty = true;
    }

    pub fn reload_series(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(series) = crate::db::series::list_series(&conn)
        {
            self.series = series;
        }
        self.dirty = true;
    }

    pub fn reload_authors(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(authors) = crate::db::documents::list_authors(&conn, 1)
        {
            self.authors = authors;
        }
        self.dirty = true;
    }

    pub fn reload_saved_searches(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(searches) = crate::db::saved_searches::list(&conn)
        {
            self.saved_searches = searches;
        }
        self.dirty = true;
    }

    pub fn reload_queue(&mut self) {
        if let Ok(conn) = self.db.lock()
            && let Ok(queue) = crate::db::documents::get_queue(&conn)
        {
            self.queue = queue;
        }
        self.dirty = true;
    }

    pub fn load_detail(&mut self) {
        if let Some(doc) = self.documents.get(self.list_cursor) {
            self.detail_doc = Some(doc.clone());
            let doc_id = doc.id.unwrap_or(0);
            if let Ok(conn) = self.db.lock() {
                self.current_notes = crate::db::notes::list(&conn, doc_id).unwrap_or_default();
                self.current_tags =
                    crate::db::documents::get_tags(&conn, doc_id).unwrap_or_default();
                self.tag_colors = crate::db::documents::get_tags_with_color(&conn)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(tag, color)| color.map(|c| (tag, c)))
                    .collect();
                self.custom_fields = crate::db::custom_fields::list_fields(&conn, doc_id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|f| (f.id, f.key, f.value))
                    .collect();
                self.current_attachments =
                    crate::db::attachments::list_for_doc(&conn, doc_id).unwrap_or_default();
            } else {
                self.current_notes = Vec::new();
                self.current_tags = Vec::new();
                self.tag_colors = HashMap::new();
                self.custom_fields = Vec::new();
                self.current_attachments = Vec::new();
            }
        }
    }

    pub fn reload_tags(&mut self) {
        if let Some(doc) = &self.detail_doc {
            let doc_id = doc.id.unwrap_or(0);
            if let Ok(conn) = self.db.lock() {
                self.current_tags =
                    crate::db::documents::get_tags(&conn, doc_id).unwrap_or_default();
                self.tag_colors = crate::db::documents::get_tags_with_color(&conn)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(tag, color)| color.map(|c| (tag, c)))
                    .collect();
            }
        }
    }

    pub fn cycle_api_mode(&mut self) {
        self.api_mode = if self.api_mode.allows_api_calls() {
            ApiMode::FullyOffline
        } else {
            ApiMode::IdentifierOnly
        };
        self.config.api_mode = self.api_mode.clone();
        self.dirty = true;
    }

    pub fn resize_left_panel(&mut self, delta: i16) {
        let max = ((self.terminal_size.0 as f32) * 0.4) as u16;
        let max = max.max(20);
        let new = (self.left_panel_width as i16 + delta).clamp(20, max as i16) as u16;
        self.left_panel_width = new;
        if let Ok(conn) = self.db.lock() {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('left_panel_width', ?1, datetime('now'))",
                rusqlite::params![new.to_string()],
            );
        }
        self.dirty = true;
    }

    pub fn reset_left_panel(&mut self) {
        self.left_panel_width = 28;
        if let Ok(conn) = self.db.lock() {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ('left_panel_width', ?1, datetime('now'))",
                rusqlite::params!["28".to_string()],
            );
        }
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

fn load_skip_delete_confirm(db: &DbConn) -> bool {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'skip_delete_confirm'",
                [],
                |row| row.get(0),
            )
            .ok();
        return v.as_deref() == Some("true");
    }
    false
}

fn load_left_panel_width(db: &DbConn) -> u16 {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'left_panel_width'",
                [],
                |row| row.get(0),
            )
            .ok();
        if let Some(s) = v {
            if let Ok(n) = s.parse::<u16>() {
                return n.clamp(20, 60);
            }
        }
    }
    28
}

fn load_metrics_backend(db: &DbConn) -> MetricsBackend {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'metrics_backend'",
                [],
                |row| row.get(0),
            )
            .ok();
        return v
            .as_deref()
            .map(MetricsBackend::parse)
            .unwrap_or(MetricsBackend::SemanticScholar);
    }
    MetricsBackend::SemanticScholar
}

fn load_openalex_api_key(db: &DbConn) -> Option<String> {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'openalex_api_key'",
                [],
                |row| row.get(0),
            )
            .ok();
        return v.filter(|s| !s.is_empty());
    }
    None
}

fn load_auto_fetch_metrics(db: &DbConn) -> bool {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'auto_fetch_metrics'",
                [],
                |row| row.get(0),
            )
            .ok();
        return v.as_deref() == Some("true");
    }
    false
}

fn load_metrics_refresh_interval_days(db: &DbConn) -> u32 {
    if let Ok(conn) = db.lock() {
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'metrics_refresh_interval_days'",
                [],
                |row| row.get(0),
            )
            .ok();
        if let Some(s) = v {
            if let Ok(days) = s.parse::<u32>() {
                if days > 0 {
                    return days;
                }
            }
        }
    }
    7
}

fn load_export_dialog_state(db: &DbConn) -> crate::export::export_dialog_state::ExportDialogState {
    use crate::citation::text::styles::{CitationLanguage, CitationStyle, DisplayMode};
    use crate::export::ExportFormat;
    use crate::export::export_dialog_state::{DialogSection, ExportDialogState};

    let default_state = ExportDialogState::new();
    if let Ok(conn) = db.lock() {
        if let Some((format, style, language)) = crate::export::preferences::load(&conn) {
            let format_cursor = ExportFormat::all()
                .iter()
                .position(|f| *f == format)
                .unwrap_or(0);
            let style_cursor = CitationStyle::all()
                .iter()
                .position(|s| *s == style)
                .unwrap_or(0);
            let language_cursor = CitationLanguage::all()
                .iter()
                .position(|l| *l == language)
                .unwrap_or(0);
            return ExportDialogState {
                selected_scope: crate::export::export_dialog_state::ExportScope::SelectedOnly,
                selected_backup_scope: crate::export::export_dialog_state::BackupScope::FullMigration,
                selected_format: format,
                selected_style: style,
                selected_language: language,
                display_mode: DisplayMode::InText,
                focused_section: DialogSection::Scope,
                scope_cursor: 0,
                backup_scope_cursor: 0,
                format_cursor,
                style_cursor,
                language_cursor,
                display_mode_cursor: 0,
                preview_text: String::new(),
            };
        }
    }
    default_state
}
