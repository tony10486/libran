pub mod library;
pub mod trash;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum FileStoragePolicy {
    #[default]
    CopyToLibrary,
    ReferenceOnly,
    CopyAndTrash,
}

impl FileStoragePolicy {
    pub fn as_str(&self) -> &str {
        match self {
            FileStoragePolicy::CopyToLibrary => "복사",
            FileStoragePolicy::ReferenceOnly => "참조",
            FileStoragePolicy::CopyAndTrash => "복사+휴지통",
        }
    }
}
