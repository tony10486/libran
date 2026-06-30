// ── Widget Manifest ────────────────────────────────────────────────────────────
//
// widget.toml 파싱. 모든 플러그인 위젯은 이 파일로 정의됩니다.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── Top-level manifest ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetManifest {
    pub widget: WidgetMeta,
    pub api: Option<ApiConfig>,
    pub script: Option<ScriptConfig>,
    pub display: Option<DisplayConfig>,
    pub permissions: Option<PermissionsConfig>,
}

impl WidgetManifest {
    /// widget.toml 파일을 파싱합니다.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: WidgetManifest = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("widget.toml 파싱 실패 ({:?}): {}", path, e))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// 매니페스트 유효성 검증.
    pub fn validate(&self) -> anyhow::Result<()> {
        use anyhow::bail;

        // id는 영숫자와 언더스코어만 허용
        let id = &self.widget.id;
        if id.is_empty() || !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            bail!("위젯 id에는 영숫자, -, _ 만 허용됩니다: {:?}", id);
        }

        match self.widget.widget_type {
            WidgetType::Api => {
                if self.api.is_none() {
                    bail!("type='api' 위젯은 [api] 섹션이 필요합니다");
                }
            }
            WidgetType::Script => {
                if self.script.is_none() {
                    bail!("type='script' 위젯은 [script] 섹션이 필요합니다");
                }
            }
            WidgetType::Builtin => {
                bail!("type='builtin'은 내부 전용입니다. api 또는 script를 사용하세요");
            }
        }

        Ok(())
    }
}

// ── [widget] meta ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetMeta {
    pub name: String,
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub widget_type: WidgetType,
    /// 자동 갱신 주기 (초). 0이면 수동 새로고침만.
    #[serde(default = "default_refresh")]
    pub refresh_interval: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 처음 설치 시 사용자에게 보안 경고 표시 여부 (script 위젯은 true 권장)
    #[serde(default)]
    pub show_security_warning: bool,
}

fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_refresh() -> u64 {
    300
}
fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    Builtin,
    Api,
    Script,
}

// ── [api] config ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_format")]
    pub response_format: ResponseFormat,
    #[serde(default)]
    pub extract: Option<ExtractConfig>,
}

fn default_method() -> String {
    "GET".to_string()
}
fn default_format() -> ResponseFormat {
    ResponseFormat::Json
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Json,
    Xml,
    Text,
}

/// 응답에서 표시할 필드를 추출하는 설정.
/// JSON은 serde_json 포인터(/path/to/field), XML은 간단한 태그명.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractConfig {
    /// JSON: ".path.to.array", XML: "//element"
    #[serde(default)]
    pub items: Option<String>,
    /// 각 항목에서 추출할 필드들
    #[serde(default)]
    pub fields: Vec<FieldExtract>,
    /// 단일 값 추출 (items 없이 사용)
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldExtract {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub default: Option<String>,
}

// ── [script] config ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// 스크립트에 전달할 추가 환경 변수 (PATH, HOME은 항상 포함)
    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ── [display] config ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 각 항목 렌더 템플릿. 플레이스홀더: {field_name}
    #[serde(default)]
    pub item_template: Option<String>,
    /// 항목 상세 줄 템플릿
    #[serde(default)]
    pub detail_template: Option<String>,
    /// 최대 표시 항목 수
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default = "default_empty")]
    pub empty_message: String,
    /// 날짜 포맷 (chrono 포맷 문자열)
    #[serde(default)]
    pub date_format: Option<String>,
}

fn default_max_items() -> usize {
    20
}
fn default_empty() -> String {
    "데이터 없음".to_string()
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            item_template: None,
            detail_template: None,
            max_items: default_max_items(),
            empty_message: default_empty(),
            date_format: None,
        }
    }
}

// ── [permissions] config ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PermissionsConfig {
    /// 이 위젯이 접근할 수 있는 도메인 목록 (글로벌 허용 목록에 합산)
    #[serde(default)]
    pub network: Vec<String>,
    /// 스크립트 실행 타임아웃 (초)
    #[serde(default = "default_exec_timeout")]
    pub max_execution_time: u64,
    /// 스크립트 stdout 최대 바이트
    #[serde(default = "default_max_output")]
    pub max_output_bytes: usize,
}

fn default_exec_timeout() -> u64 {
    10
}
fn default_max_output() -> usize {
    65536
}
