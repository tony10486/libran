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
}
