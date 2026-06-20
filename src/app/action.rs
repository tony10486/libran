use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::api::ApiMode;
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

    StartCitationExtraction { doc_id: i64 },
    CitationExtracted { doc_id: i64, edge_count: usize, unmatched_count: usize },
    CitationExtractionFailed { doc_id: i64, reason: String },

    StartManualCitationEntry { doc_id: i64 },
    ManualCitationSaved { source_id: i64, target_id: i64 },
    StartBibtexImport { doc_id: i64, path: String },
    BibtexImported { doc_id: i64, entry_count: usize },

    GenerateCitationGraph { doc_ids: Vec<i64> },
    CitationGraphReady { graph_state: Box<crate::app::graph_state::GraphState>, cache_key: String, cache_hit: bool },
    ToggleGraphRenderMode,
    NavigateGraph { direction: GraphDirection },
    SelectGraphNode { node_idx: usize },
    ExitGraphView,

    MouseHover { column: u16, row: u16 },
    MouseClick { column: u16, row: u16 },
    TerminalResize { width: u16, height: u16 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphDirection {
    Up,
    Down,
    Left,
    Right,
}
