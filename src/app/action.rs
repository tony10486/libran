use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::api::ApiMode;
use crate::api::metrics::{AuthorMetrics, MetricsBackend};
use crate::export::ExportFormat;
use crate::pdf::RawMetadata;

#[derive(Clone, Debug)]
pub enum AppAction {
    Tick,
    KeyPressed(KeyEvent),
    DragDetected(PathBuf),
    StartMetadataExtraction(PathBuf),
    MetadataExtracted(Box<RawMetadata>, PathBuf),
    MetadataSaved(i64),
    ApiLookupSuccess(RawMetadata, i64),
    ApiLookupFailed(String),
    ApiLookupSkipped(String),
    ToggleApiMode,
    ApiModeChanged(ApiMode),
    UpdateSearchFilter(String),
    SelectProject(Option<i64>),
    CreateProject(String),
    ToggleClassificationScheme(String),
    ExportRequested(ExportFormat),
    UpdateDocument(i64, Box<crate::db::documents::Document>),
    DeleteDocument(i64),
    SaveConfig,
    OperationFailed(String),
    SystemShutdown,
    SortBySimilarity(i64),
    ClearSimilaritySort,

    StartCitationExtraction {
        doc_id: i64,
    },
    CitationExtracted {
        doc_id: i64,
        edge_count: usize,
        unmatched_count: usize,
    },
    CitationExtractionFailed {
        doc_id: i64,
        reason: String,
    },

    StartManualCitationEntry {
        doc_id: i64,
    },
    ManualCitationSaved {
        source_id: i64,
        target_id: i64,
    },
    StartBibtexImport {
        doc_id: i64,
        path: String,
    },
    BibtexImported {
        doc_id: i64,
        entry_count: usize,
    },

    GenerateCitationGraph {
        doc_ids: Vec<i64>,
    },
    CitationGraphReady {
        graph_state: Box<crate::app::graph_state::GraphState>,
        cache_key: String,
        cache_hit: bool,
    },
    ToggleGraphRenderMode,
    NavigateGraph {
        direction: GraphDirection,
    },
    SelectGraphNode {
        node_idx: usize,
    },
    ExitGraphView,

    MouseHover {
        column: u16,
        row: u16,
    },
    MouseClick {
        column: u16,
        row: u16,
    },
    TerminalResize {
        width: u16,
        height: u16,
    },

    AddTag {
        doc_id: i64,
        tag: String,
    },
    RemoveTag {
        doc_id: i64,
        tag: String,
    },
    SetRating {
        doc_id: i64,
        rating: Option<u8>,
    },

    CreateSeries(String),
    SelectSeries(Option<i64>),
    DeleteSeries(i64),
    ToggleSeriesGrouping,
    AssignDocToSeries {
        doc_id: i64,
        series_id: i64,
        volume: Option<String>,
        issue: Option<String>,
    },
    AutoGroupSeries,

    AddDocsToProject {
        project_id: i64,
        doc_ids: Vec<i64>,
    },
    DeleteProject(i64),

    SelectAuthor(Option<String>),

    FetchAuthorMetrics {
        name: String,
    },
    AuthorMetricsFetched {
        name: String,
        metrics: Box<AuthorMetrics>,
    },
    AuthorMetricsFailed {
        name: String,
        reason: String,
    },
    SetMetricsBackend(MetricsBackend),
    RegisterApiKey(String),
    ShowMetricsOverlay {
        name: String,
    },
    CloseMetricsOverlay,

    LookupByDoi {
        doc_id: i64,
    },

    SelectUdc(Option<String>),

    AddCustomField {
        doc_id: i64,
        key: String,
        value: String,
    },
    DeleteCustomField {
        doc_id: i64,
        field_id: i64,
    },

    OpenExternalViewer {
        doc_id: i64,
    },
    OpenExternalViewerResult {
        success: bool,
        message: String,
    },

    // ── Reading status (H) ──
    ToggleReadingStatus {
        doc_id: i64,
    },

    // ── Saved searches (G) ──
    SaveCurrentSearch,
    SaveCurrentSearchNamed {
        name: String,
    },
    SelectSavedSearch {
        search_id: i64,
    },
    DeleteSavedSearch {
        search_id: i64,
    },

    // ── Statistics dashboard (I) ──
    ToggleStatsDashboard,

    // ── PDF bookmarks (J) ──
    ExtractBookmarks {
        doc_id: i64,
    },
    BookmarksExtracted {
        doc_id: i64,
        bookmarks: Vec<(String, i64)>,
    },
    BookmarkExtractionFailed {
        doc_id: i64,
        reason: String,
    },

    // ── Bulk DOI import (C) ──
    StartBulkImport,
    BulkImportSubmitted(String),
    BulkImportResult {
        success_count: usize,
        fail_count: usize,
        message: String,
    },

    // ── File import (F) ──
    StartFileImport,
    FileImportSubmitted(String),
    FileImportResult {
        count: usize,
        message: String,
    },

    // ── Forward citations (B) ──
    FetchForwardCitations {
        doc_id: i64,
    },
    ForwardCitationsFetched {
        doc_id: i64,
        count: i64,
    },
    ForwardCitationsFailed {
        doc_id: i64,
        reason: String,
    },

    // ── Author merge (E) ──
    StartAuthorMerge,
    AuthorMergeSourceEntered(String),
    AuthorMergeCanonicalEntered {
        source: String,
        canonical: String,
    },
    AuthorMergeResult {
        success: bool,
        message: String,
    },

    // ── Backup/restore (:backup / :restore) ──
    Backup {
        path: String,
    },
    Restore {
        path: String,
    },

    // ── Favorite filter (*) ──
    ToggleFavoriteFilter,

    // ── Tag color (:tag-color <tag> <hex>) ──
    SetTagColor {
        tag: String,
        color: Option<String>,
    },

    // ── Classification CSV import (:import-classification <path>) ──
    ImportClassification {
        path: String,
    },

    // ── Reading queue / TBR (Q/R/Y) ──
    ToggleQueueView,
    AddToQueue {
        doc_id: i64,
    },
    RemoveFromQueue {
        doc_id: i64,
    },
    ReorderQueue {
        doc_id: i64,
        new_position: usize,
    },

    // ── Reading progress (>/<) ──
    UpdateReadingProgress {
        doc_id: i64,
        progress: i64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphDirection {
    Up,
    Down,
    Left,
    Right,
}
