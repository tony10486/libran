// ── API Widget Runner ──────────────────────────────────────────────────────────
//
// Declarative API 위젯 실행기.
// widget.toml의 [api] 섹션을 기반으로 HTTP 요청을 수행하고
// 응답을 템플릿으로 변환하여 WOP WidgetContent를 생성합니다.

use std::collections::HashMap;
use std::time::Instant;

use tracing::warn;

use crate::widget::manifest::{ApiConfig, DisplayConfig, ResponseFormat, WidgetMeta};
use crate::widget::sandbox::Sandbox;
use crate::widget::{WidgetContent, WidgetLine, WidgetStatus};

pub struct ApiWidgetRunner {
    pub(crate) meta: WidgetMeta,
    pub(crate) api_cfg: ApiConfig,
    pub(crate) display_cfg: DisplayConfig,
    pub(crate) sandbox: Sandbox,
    cached_content: WidgetContent,
    last_refresh: Option<Instant>,
}

impl ApiWidgetRunner {
    pub fn new(
        meta: WidgetMeta,
        api_cfg: ApiConfig,
        display_cfg: DisplayConfig,
        sandbox: Sandbox,
    ) -> Self {
        ApiWidgetRunner {
            meta,
            api_cfg,
            display_cfg,
            sandbox,
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

    /// last_refresh를 현재 시각으로 갱신합니다 (fetch 완료 후 호출).
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Some(Instant::now());
    }

    /// 비동기 데이터 fetch 후 캐시를 갱신합니다.
    /// main loop에서 tokio::spawn으로 실행됩니다.
    pub async fn fetch_and_update(&mut self) {
        self.cached_content = WidgetContent::loading();
        let result = self.fetch().await;
        self.cached_content = match result {
            Ok(content) => content,
            Err(e) => {
                warn!("API 위젯 [{}] fetch 오류: {}", self.meta.id, e);
                WidgetContent::error(format!("API 오류: {}", e))
            }
        };
        self.last_refresh = Some(Instant::now());
    }

    pub(crate) async fn fetch(&self) -> anyhow::Result<WidgetContent> {
        let body = self
            .sandbox
            .http_get(&self.api_cfg.url, Some(&self.api_cfg.headers))
            .await?;

        let lines = match self.api_cfg.response_format {
            ResponseFormat::Json => self.parse_json(&body),
            ResponseFormat::Xml => self.parse_xml(&body),
            ResponseFormat::Text => self.parse_text(&body),
        }?;

        Ok(WidgetContent {
            version: 1,
            status: WidgetStatus::Ok,
            lines,
            ..Default::default()
        })
    }

    // ── Response parsers ───────────────────────────────────────────────────────

    fn parse_json(&self, body: &str) -> anyhow::Result<Vec<WidgetLine>> {
        let value: serde_json::Value = serde_json::from_str(body)?;
        let max = self.display_cfg.max_items;
        let item_tpl = self
            .display_cfg
            .item_template
            .as_deref()
            .unwrap_or("{value}");
        let detail_tpl = self.display_cfg.detail_template.as_deref();

        let mut lines = Vec::new();

        // 배열 또는 단일 값 처리
        if let Some(extract) = &self.api_cfg.extract {
            if let Some(items_path) = &extract.items {
                // JSON Pointer로 배열 찾기 (/path/to/array 형식)
                let arr_val = if items_path.starts_with('/') {
                    value.pointer(items_path)
                } else {
                    value.pointer(&format!("/{}", items_path.replace('.', "/")))
                };

                if let Some(serde_json::Value::Array(arr)) = arr_val {
                    for (i, item) in arr.iter().enumerate().take(max) {
                        let fields = self.extract_json_fields(item, &extract.fields);
                        lines.push(WidgetLine::new(apply_template(item_tpl, &fields)));
                        if let Some(dtpl) = detail_tpl {
                            lines.push(WidgetLine::new(apply_template(dtpl, &fields)).dim());
                        }
                        if i + 1 < arr.len().min(max) {
                            lines.push(WidgetLine::new("").dim());
                        }
                    }
                } else {
                    lines.push(WidgetLine::new(&self.display_cfg.empty_message).dim());
                }
            } else if let Some(val_path) = &extract.value {
                let ptr = if val_path.starts_with('/') {
                    val_path.clone()
                } else {
                    format!("/{}", val_path.replace('.', "/"))
                };
                let val = value.pointer(&ptr).and_then(|v| v.as_str()).unwrap_or("");
                lines.push(WidgetLine::new(val));
            }
        } else {
            // extract 없이 최상위 배열로 처리
            if let serde_json::Value::Array(arr) = &value {
                for item in arr.iter().take(max) {
                    let text = item.as_str().unwrap_or(&item.to_string()).to_string();
                    lines.push(WidgetLine::new(apply_template(item_tpl, &[("value", text)])));
                }
            } else {
                lines.push(WidgetLine::new(body.lines().next().unwrap_or("")).dim());
            }
        }

        if lines.is_empty() {
            lines.push(WidgetLine::new(&self.display_cfg.empty_message).dim());
        }

        Ok(lines)
    }

    fn extract_json_fields(
        &self,
        item: &serde_json::Value,
        fields: &[crate::widget::manifest::FieldExtract],
    ) -> Vec<(String, String)> {
        fields
            .iter()
            .map(|f| {
                let ptr = if f.path.starts_with('/') {
                    f.path.clone()
                } else {
                    format!("/{}", f.path.replace('.', "/"))
                };
                let val = item
                    .pointer(&ptr)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| f.default.clone())
                    .unwrap_or_default();
                (f.name.clone(), val)
            })
            .collect()
    }

    fn parse_xml(&self, body: &str) -> anyhow::Result<Vec<WidgetLine>> {
        // 간단한 XML 파싱: 태그 텍스트 추출 (quick-xml)
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let item_tpl = self
            .display_cfg
            .item_template
            .as_deref()
            .unwrap_or("{title}");
        let detail_tpl = self.display_cfg.detail_template.as_deref();
        let max = self.display_cfg.max_items;

        let mut reader = Reader::from_str(body);
        reader.config_mut().trim_text(true);

        let mut lines = Vec::new();
        let mut current_fields: HashMap<String, String> = HashMap::new();
        let mut current_tag = String::new();
        let mut in_item = false;
        let mut item_count = 0;

        let fields_of_interest: Vec<String> = if let Some(extract) = &self.api_cfg.extract {
            extract.fields.iter().map(|f| f.name.clone()).collect()
        } else {
            vec!["title".to_string(), "author".to_string(), "published".to_string()]
        };

        let item_tag = self
            .api_cfg
            .extract
            .as_ref()
            .and_then(|e| e.items.as_ref())
            .map(|s| s.trim_matches('/').to_string())
            .unwrap_or_else(|| "entry".to_string());

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == item_tag {
                        in_item = true;
                        current_fields.clear();
                    }
                    current_tag = tag;
                }
                Ok(Event::Text(e)) => {
                    if in_item && fields_of_interest.contains(&current_tag) {
                        let text = e.unescape().unwrap_or_default().to_string();
                        if !text.is_empty() {
                            current_fields.entry(current_tag.clone()).or_insert(text);
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == item_tag && in_item {
                        if item_count < max {
                            let fields: Vec<(String, String)> = current_fields
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            lines.push(WidgetLine::new(apply_template(item_tpl, &fields)));
                            if let Some(dtpl) = detail_tpl {
                                lines.push(WidgetLine::new(apply_template(dtpl, &fields)).dim());
                            }
                        }
                        item_count += 1;
                        in_item = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        if lines.is_empty() {
            lines.push(WidgetLine::new(&self.display_cfg.empty_message).dim());
        }

        Ok(lines)
    }

    fn parse_text(&self, body: &str) -> anyhow::Result<Vec<WidgetLine>> {
        let max = self.display_cfg.max_items;
        let lines = body
            .lines()
            .take(max)
            .map(|l| WidgetLine::new(l))
            .collect();
        Ok(lines)
    }
}

// ── Template engine ────────────────────────────────────────────────────────────

/// {field_name} 플레이스홀더를 실제 값으로 치환합니다.
fn apply_template<S: AsRef<str>, T: AsRef<str>>(template: &str, fields: &[(S, T)]) -> String {
    let mut result = template.to_string();
    for (name, value) in fields {
        let placeholder = format!("{{{}}}", name.as_ref());
        result = result.replace(&placeholder, value.as_ref());
    }
    result
}
