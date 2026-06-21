pub mod arxiv;
pub mod cache;
pub mod crossref;
pub mod metrics;
pub mod rate_limiter;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum ApiMode {
    #[default]
    IdentifierOnly,
    ManualSearch,
    AutoFallback,
    FullyOffline,
}

impl ApiMode {
    pub fn as_str(&self) -> &str {
        match self {
            ApiMode::IdentifierOnly => "식별자만",
            ApiMode::ManualSearch => "수동 검색",
            ApiMode::AutoFallback => "자동 폴백",
            ApiMode::FullyOffline => "오프라인",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "identifier" => ApiMode::IdentifierOnly,
            "manual" => ApiMode::ManualSearch,
            "auto" => ApiMode::AutoFallback,
            _ => ApiMode::FullyOffline,
        }
    }

    pub fn allows_api_calls(&self) -> bool {
        !matches!(self, ApiMode::FullyOffline)
    }
}
