// ── Widget Discovery ───────────────────────────────────────────────────────────
//
// ~/.libran/widgets/*/widget.toml 을 스캔하여 플러그인 위젯을 찾습니다.

use std::path::PathBuf;

use tracing::{info, warn};

use crate::widget::manifest::{WidgetManifest, WidgetType};
use crate::widget::{WidgetInstance, WidgetRegistry};
use crate::widget::api_runner::ApiWidgetRunner;
use crate::widget::sandbox::{Sandbox, make_widget_sandbox};
use crate::widget::script_runner::ScriptWidgetRunner;

/// 위젯 디렉터리 경로를 반환합니다.
pub fn widgets_dir() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".libran").join("widgets"))
        .unwrap_or_else(|| PathBuf::from(".libran/widgets"))
}

/// 위젯 디렉터리 내 모든 widget.toml을 스캔하여 로드 결과를 반환합니다.
/// 실패한 위젯은 경고 로그를 남기고 건너뜁니다.
pub fn discover_plugin_widgets(
    registry: &mut WidgetRegistry,
    global_sandbox: &Sandbox,
) {
    let dir = widgets_dir();
    if !dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            warn!("위젯 디렉터리 생성 실패: {}", e);
        }
        return;
    }

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("위젯 디렉터리 읽기 실패: {}", e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("widget.toml");
        if !manifest_path.exists() {
            continue;
        }

        match load_plugin_widget(&manifest_path, &path, global_sandbox) {
            Ok(instance) => {
                info!("위젯 로드: {} ({})", instance.name(), instance.id());
                registry.push(instance);
            }
            Err(e) => {
                warn!("위젯 로드 실패 ({:?}): {}", manifest_path, e);
            }
        }
    }
}

fn load_plugin_widget(
    manifest_path: &PathBuf,
    widget_dir: &PathBuf,
    global_sandbox: &Sandbox,
) -> anyhow::Result<WidgetInstance> {
    let manifest = WidgetManifest::load(manifest_path)?;

    if !manifest.widget.enabled {
        anyhow::bail!("위젯 비활성화됨: {}", manifest.widget.id);
    }

    // 위젯별 도메인 권한 합산
    let widget_domains = manifest
        .permissions
        .as_ref()
        .map(|p| p.network.clone())
        .unwrap_or_default();
    let sandbox = make_widget_sandbox(global_sandbox, &widget_domains);

    match manifest.widget.widget_type {
        WidgetType::Api => {
            let api_cfg = manifest.api.clone().ok_or_else(|| {
                anyhow::anyhow!("api 위젯에 [api] 섹션 없음")
            })?;
            let display_cfg = manifest.display.clone().unwrap_or_default();
            let runner = ApiWidgetRunner::new(manifest.widget.clone(), api_cfg, display_cfg, sandbox);
            Ok(WidgetInstance::Api(runner))
        }
        WidgetType::Script => {
            let script_cfg = manifest.script.clone().ok_or_else(|| {
                anyhow::anyhow!("script 위젯에 [script] 섹션 없음")
            })?;
            let perms = manifest.permissions.clone().unwrap_or_default();
            let runner = ScriptWidgetRunner::new(
                manifest.widget.clone(),
                script_cfg,
                widget_dir.clone(),
                perms,
            );
            Ok(WidgetInstance::Script(runner))
        }
        WidgetType::Builtin => {
            anyhow::bail!("builtin 타입은 플러그인에서 사용 불가")
        }
    }
}

/// 위젯 예시 파일을 생성합니다 (처음 실행 시 도움말용).
pub fn write_example_widgets() {
    let dir = widgets_dir();
    let _ = std::fs::create_dir_all(&dir);

    // arXiv 예시 위젯
    let arxiv_dir = dir.join("arxiv_example");
    if !arxiv_dir.exists() {
        let _ = std::fs::create_dir_all(&arxiv_dir);
        let toml = r#"[widget]
name = "arXiv CS.AI"
id = "arxiv_ai"
version = "1.0.0"
description = "최신 arXiv CS.AI 논문 피드"
type = "api"
refresh_interval = 1800
enabled = false

[api]
url = "https://export.arxiv.org/api/query?search_query=cat:cs.AI&max_results=5&sortBy=submittedDate&sortOrder=descending"
method = "GET"
response_format = "xml"

[display]
item_template = "📄 {title}"
detail_template = "   👤 {author}"
max_items = 5
empty_message = "논문 없음"

[permissions]
network = ["export.arxiv.org"]
"#;
        let _ = std::fs::write(arxiv_dir.join("widget.toml"), toml);
    }

    // Python 스크립트 예시 위젯
    let sysmon_dir = dir.join("sysmon_example");
    if !sysmon_dir.exists() {
        let _ = std::fs::create_dir_all(&sysmon_dir);
        let toml = r#"[widget]
name = "System Monitor"
id = "sysmon"
version = "1.0.0"
description = "CPU/메모리 사용률 표시"
type = "script"
refresh_interval = 5
enabled = false
show_security_warning = true

[script]
command = "python3"
args = ["monitor.py"]

[permissions]
network = []
max_execution_time = 5
max_output_bytes = 65536
"#;
        let monitor_py = r#"#!/usr/bin/env python3
# Widget Output Protocol (WOP) 예시
# 이 스크립트는 JSON을 stdout에 출력해야 합니다.
import json

output = {
    "version": 1,
    "status": "ok",
    "title": "System Monitor",
    "lines": [
        {"text": "이 예시 스크립트는 실제 시스템 정보를 읽지 않습니다.", "style": "dim"},
        {"text": "psutil 등을 활용해 실제 데이터를 출력하세요.", "style": "normal"},
        {"text": "", "style": "normal"},
        {"text": "CPU:  --", "style": "bold"},
        {"text": "MEM:  --", "style": "bold"},
    ],
    "actions": [
        {"key": "r", "label": "새로고침", "action": "refresh"}
    ]
}

print(json.dumps(output))
"#;
        let _ = std::fs::write(sysmon_dir.join("widget.toml"), toml);
        let _ = std::fs::write(sysmon_dir.join("monitor.py"), monitor_py);
    }
}
