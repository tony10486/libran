use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum AppAction {
    Tick,
    KeyPressed(char),
    DragDetected(PathBuf),
    StartMetadataExtraction(PathBuf),
    MetadataExtracted(Box<crate::pdf::RawMetadata>),
    ApiLookupSuccess(String),
    ApiLookupFailed(String),
    ApiLookupSkipped(String),
    UpdateSearchFilter(String),
    SelectProject(Option<i64>),
    ToggleClassificationScheme(String),
    OperationFailed(String),
    SystemShutdown,
}
