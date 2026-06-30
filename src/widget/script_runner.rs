// ── Script Widget Runner ───────────────────────────────────────────────────────
//
// 외부 스크립트를 실행하고 WOP JSON 출력을 파싱합니다.
// 보안: 타임아웃, stdout 크기 제한, 환경변수 최소화, CWD 격리.

use std::path::PathBuf;
use std::time::Instant;

use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::warn;

use crate::widget::manifest::{PermissionsConfig, ScriptConfig, WidgetMeta};
use crate::widget::WidgetContent;

pub struct ScriptWidgetRunner {
    pub(crate) meta: WidgetMeta,
    pub(crate) script_cfg: ScriptConfig,
    pub(crate) widget_dir: PathBuf,
    pub(crate) permissions: PermissionsConfig,
    cached_content: WidgetContent,
    last_refresh: Option<Instant>,
}

impl ScriptWidgetRunner {
    pub fn new(
        meta: WidgetMeta,
        script_cfg: ScriptConfig,
        widget_dir: PathBuf,
        permissions: PermissionsConfig,
    ) -> Self {
        ScriptWidgetRunner {
            meta,
            script_cfg,
            widget_dir,
            permissions,
            cached_content: WidgetContent::loading(),
            last_refresh: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.meta.id
    }

    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn refresh_interval_secs(&self) -> u64 {
        self.meta.refresh_interval
    }

    pub fn content(&self) -> &WidgetContent {
        &self.cached_content
    }

    /// 위젯 바용 컴팩트 요약 (cached_content의 badge 사용).
    pub fn compact_bar(&self) -> Option<String> {
        self.cached_content.badge.clone()
    }

    pub fn needs_refresh(&self) -> bool {
        match self.last_refresh {
            None => true,
            Some(t) => t.elapsed().as_secs() >= self.meta.refresh_interval,
        }
    }

    /// last_refresh를 현재 시각으로 갱신합니다 (run 완료 후 호출).
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Some(Instant::now());
    }

    /// 스크립트를 실행하고 WOP JSON을 파싱하여 캐시를 갱신합니다.
    pub async fn run_and_update(&mut self) {
        self.cached_content = WidgetContent::loading();
        let result = self.run().await;
        self.cached_content = match result {
            Ok(content) => content,
            Err(e) => {
                warn!("스크립트 위젯 [{}] 실행 오류: {}", self.meta.id, e);
                WidgetContent::error(format!("스크립트 오류: {}", e))
            }
        };
        self.last_refresh = Some(Instant::now());
    }

    pub(crate) async fn run(&self) -> anyhow::Result<WidgetContent> {
        let timeout = tokio::time::Duration::from_secs(self.permissions.max_execution_time);
        let max_output = self.permissions.max_output_bytes;

        // 환경변수 최소화 (PATH, HOME, LANG, 사용자 정의만)
        let mut cmd = Command::new(&self.script_cfg.command);
        cmd.args(&self.script_cfg.args)
            .current_dir(&self.widget_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            // 환경변수 초기화 후 최소 세트만 허용
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()))
            .env("HOME", std::env::var("HOME").unwrap_or_default())
            .env("LANG", std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string()));

        // 사용자 정의 환경변수 추가
        for (k, v) in &self.script_cfg.env {
            // PATH, HOME 오버라이드 방지
            let key_lower = k.to_lowercase();
            if key_lower != "path" && key_lower != "home" {
                cmd.env(k, v);
            }
        }

        let mut child = cmd.spawn()?;
        let mut stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("stdout 없음"))?;

        // 타임아웃과 함께 프로세스 실행
        let result = tokio::time::timeout(timeout, async {
            // 크기 제한을 두고 읽기
            let mut buf = Vec::with_capacity(4096);
            let mut temp = [0u8; 4096];
            loop {
                match stdout.read(&mut temp).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if buf.len() + n > max_output {
                            return Err(anyhow::anyhow!(
                                "스크립트 출력 크기 초과 (최대 {} bytes)",
                                max_output
                            ));
                        }
                        buf.extend_from_slice(&temp[..n]);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Ok(buf)
        })
        .await;

        // 프로세스 정리
        let _ = child.kill().await;

        let output_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "스크립트 실행 타임아웃 ({}초)",
                    self.permissions.max_execution_time
                ));
            }
        };

        let output_str = String::from_utf8_lossy(&output_bytes);
        self.parse_wop_output(&output_str)
    }

    fn parse_wop_output(&self, output: &str) -> anyhow::Result<WidgetContent> {
        // WOP JSON 파싱
        let trimmed = output.trim();
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("스크립트 출력이 비어있습니다"));
        }

        let content: WidgetContent = serde_json::from_str(trimmed)
            .map_err(|e| anyhow::anyhow!("WOP JSON 파싱 실패: {} (출력: {:?})", e, &trimmed[..trimmed.len().min(200)]))?;

        // 프로토콜 버전 확인
        if content.version != 1 {
            return Err(anyhow::anyhow!(
                "지원하지 않는 WOP 버전: {} (지원: 1)",
                content.version
            ));
        }

        // action 키 유효성 검사 (단일 ASCII 문자만 허용)
        for action in &content.actions {
            if !action.key.is_ascii_alphanumeric() {
                return Err(anyhow::anyhow!(
                    "위젯 액션 키는 영숫자여야 합니다: {:?}",
                    action.key
                ));
            }
        }

        Ok(content)
    }
}
