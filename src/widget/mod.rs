// ── Widget Engine ─────────────────────────────────────────────────────────────
//
// Plugin-based widget architecture:
//   Tier 1 – API    : Declarative HTTP widgets defined in widget.toml
//   Tier 2 – Script : External-process widgets that emit WOP JSON
//
// All widgets are loaded from ~/.libran/widgets/<name>/widget.toml.
// The core binary contains no built-in widgets — only the loading infrastructure.
// All widgets communicate through the Widget Output Protocol (WOP).

pub mod api_runner;
pub mod discovery;
pub mod manifest;
pub mod sandbox;
pub mod script_runner;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// ── Widget Output Protocol (WOP) ──────────────────────────────────────────────

/// 위젯의 렌더링 상태.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetStatus {
    Ok,
    Error,
    Loading,
}

impl Default for WidgetStatus {
    fn default() -> Self {
        WidgetStatus::Loading
    }
}

/// 위젯 한 줄의 스타일 프리셋.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    #[default]
    Normal,
    Bold,
    Dim,
    Italic,
    Highlight,
    Error,
    Success,
    Warning,
}

/// 텍스트 정렬.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// WOP의 단일 텍스트 라인.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WidgetLine {
    pub text: String,
    #[serde(default)]
    pub style: LineStyle,
    /// 선택적 16진수 색상 코드 ("RRGGBB" 또는 "#RRGGBB")
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub align: TextAlign,
    /// 텍스트 앞에 붙이는 유니코드 아이콘
    #[serde(default)]
    pub icon: Option<String>,
}

impl WidgetLine {
    pub fn new(text: impl Into<String>) -> Self {
        WidgetLine {
            text: text.into(),
            ..Default::default()
        }
    }
    pub fn bold(mut self) -> Self {
        self.style = LineStyle::Bold;
        self
    }
    pub fn dim(mut self) -> Self {
        self.style = LineStyle::Dim;
        self
    }
    pub fn highlight(mut self) -> Self {
        self.style = LineStyle::Highlight;
        self
    }
    pub fn warning(mut self) -> Self {
        self.style = LineStyle::Warning;
        self
    }
    pub fn error_style(mut self) -> Self {
        self.style = LineStyle::Error;
        self
    }
    pub fn success(mut self) -> Self {
        self.style = LineStyle::Success;
        self
    }
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn center(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }
    pub fn color(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(hex.into());
        self
    }
}

/// 논리적으로 묶인 줄 그룹.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WidgetSection {
    pub header: String,
    pub lines: Vec<WidgetLine>,
}

/// 사용자가 위젯 내에서 실행할 수 있는 커스텀 액션.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetActionDef {
    /// 단축키 (단일 문자)
    pub key: char,
    pub label: String,
    /// "refresh" | "open_url" | "custom"
    pub action: String,
    /// action="open_url" 시 URL, action="custom" 시 임의 페이로드
    #[serde(default)]
    pub payload: Option<String>,
}

/// Widget Output Protocol — 모든 위젯이 반환하는 단일 출력 타입.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WidgetContent {
    pub version: u32,
    #[serde(default)]
    pub status: WidgetStatus,
    /// 오버라이드 제목 (없으면 manifest의 name 사용)
    #[serde(default)]
    pub title: Option<String>,
    /// 본문 텍스트 라인 (sections와 함께 쓰거나 단독으로 사용)
    #[serde(default)]
    pub lines: Vec<WidgetLine>,
    /// 논리 섹션 (lines 대신 또는 추가로 사용)
    #[serde(default)]
    pub sections: Vec<WidgetSection>,
    /// 위젯 패널 탭에 표시할 뱃지 텍스트
    #[serde(default)]
    pub badge: Option<String>,
    /// 오류 상세 메시지
    #[serde(default)]
    pub error_message: Option<String>,
    /// 다음 갱신까지 초 (widget.toml의 refresh_interval 오버라이드)
    #[serde(default)]
    pub next_refresh: Option<u64>,
    /// 위젯이 제공하는 커스텀 액션
    #[serde(default)]
    pub actions: Vec<WidgetActionDef>,
}

impl WidgetContent {
    pub fn loading() -> Self {
        WidgetContent {
            version: 1,
            status: WidgetStatus::Loading,
            lines: vec![WidgetLine::new("  ⏳ 로딩 중...").dim()],
            ..Default::default()
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        WidgetContent {
            version: 1,
            status: WidgetStatus::Error,
            error_message: Some(msg.into()),
            ..Default::default()
        }
    }

    pub fn simple(lines: Vec<WidgetLine>) -> Self {
        WidgetContent {
            version: 1,
            status: WidgetStatus::Ok,
            lines,
            ..Default::default()
        }
    }
}

// ── WidgetInstance ────────────────────────────────────────────────────────────

/// 두 유형의 위젯을 하나의 enum으로 통합.
/// Builtin 타입은 코어에서 제거되었으며, 모든 위젯은 플러그인(API/Script)으로 로드됩니다.
pub enum WidgetInstance {
    Api(api_runner::ApiWidgetRunner),
    Script(script_runner::ScriptWidgetRunner),
}

impl WidgetInstance {
    pub fn id(&self) -> &str {
        match self {
            WidgetInstance::Api(r) => r.id(),
            WidgetInstance::Script(r) => r.id(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            WidgetInstance::Api(r) => r.name(),
            WidgetInstance::Script(r) => r.name(),
        }
    }

    pub fn refresh_interval_secs(&self) -> u64 {
        match self {
            WidgetInstance::Api(r) => r.refresh_interval_secs(),
            WidgetInstance::Script(r) => r.refresh_interval_secs(),
        }
    }

    /// 현재 캐시된 콘텐츠를 반환합니다.
    pub fn content(&self) -> &WidgetContent {
        match self {
            WidgetInstance::Api(r) => r.content(),
            WidgetInstance::Script(r) => r.content(),
        }
    }

    /// 사이드바 상단 위젯 바에 표시할 컴팩트 요약.
    pub fn compact_bar(&self) -> Option<String> {
        match self {
            WidgetInstance::Api(r) => r.compact_bar(),
            WidgetInstance::Script(r) => r.compact_bar(),
        }
    }
}

// ── WidgetRegistry ────────────────────────────────────────────────────────────

/// 위젯 인스턴스와 캐시된 콘텐츠를 함께 관리하는 레지스트리.
pub struct WidgetRegistry {
    pub widgets: Vec<WidgetInstance>,
    /// 각 위젯의 현재 콘텐츠 캐시
    pub contents: Vec<WidgetContent>,
    pub active_index: usize,
    /// 보안 경고를 수락한 위젯 ID 집합 (show_security_warning = true인 스크립트 위젯용).
    /// 수락되기 전까지 해당 위젯의 스크립트는 실행되지 않음.
    pub security_approved: HashSet<String>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        WidgetRegistry {
            widgets: Vec::new(),
            contents: Vec::new(),
            active_index: 0,
            security_approved: HashSet::new(),
        }
    }

    /// 보안 경고를 수락하여 위젯의 스크립트 실행을 허용합니다.
    pub fn acknowledge_security_warning(&mut self, widget_id: &str) {
        self.security_approved.insert(widget_id.to_string());
    }

    /// 위젯의 보안 경고가 수락되었는지 확인합니다.
    /// show_security_warning가 true인 위젯만 체크하며, false인 위젯은 항상 허용됩니다.
    pub fn is_security_approved(&self, widget_id: &str) -> bool {
        self.widgets
            .iter()
            .find(|w| w.id() == widget_id)
            .map(|w| match w {
                WidgetInstance::Script(runner) => {
                    if runner.meta.show_security_warning {
                        self.security_approved.contains(widget_id)
                    } else {
                        true // 보안 경고가 필요 없는 위젯은 항상 허용
                    }
                }
                WidgetInstance::Api(_) => true, // API 위젯은 스크립트 실행 없음
            })
            .unwrap_or(false)
    }

    pub fn push(&mut self, instance: WidgetInstance) {
        self.contents.push(WidgetContent::loading());
        self.widgets.push(instance);
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    pub fn active_widget(&self) -> Option<&WidgetInstance> {
        self.widgets.get(self.active_index)
    }

    pub fn active_widget_mut(&mut self) -> Option<&mut WidgetInstance> {
        self.widgets.get_mut(self.active_index)
    }

    pub fn active_content(&self) -> Option<&WidgetContent> {
        self.contents.get(self.active_index)
    }

    pub fn set_content(&mut self, widget_id: &str, content: WidgetContent) {
        if let Some(idx) = self.widgets.iter().position(|w| w.id() == widget_id) {
            if idx < self.contents.len() {
                self.contents[idx] = content;
            }
        }
    }

    /// 모든 위젯의 이름과 뱃지를 반환 (탭 바 렌더용).
    pub fn tab_labels(&self) -> Vec<(String, Option<String>)> {
        self.widgets
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let badge = self.contents.get(i).and_then(|c| c.badge.clone());
                (w.name().to_string(), badge)
            })
            .collect()
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.widgets.len() {
            self.active_index = index;
        }
    }

    pub fn next_tab(&mut self) {
        if !self.widgets.is_empty() {
            self.active_index = (self.active_index + 1) % self.widgets.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.widgets.is_empty() {
            self.active_index = (self.active_index + self.widgets.len() - 1) % self.widgets.len();
        }
    }

    /// 모든 위젯의 컴팩트 바 요약을 반환 (위젯 바 렌더용).
    /// None인 위젯은 건너뜁니다.
    pub fn compact_bars(&self) -> Vec<String> {
        self.widgets
            .iter()
            .filter_map(|w| w.compact_bar())
            .collect()
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
