// ── Built-in Schedule Widget ───────────────────────────────────────────────────
//
// 로컬 JSON 파일 기반 일정 관리 위젯.
// 데이터 파일: ~/.libran/widgets/schedule.json

use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};

use crate::widget::sandbox::Sandbox;
use crate::widget::{
    BuiltinWidget, WidgetActionDef, WidgetContent, WidgetKeyResult, WidgetLine, WidgetStatus,
};

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: u64,
    pub title: String,
    /// ISO 8601 날짜 문자열 (YYYY-MM-DD), 없으면 날짜 없음
    pub due_date: Option<String>,
    pub done: bool,
    pub created_at: String,
}

#[derive(Default, Serialize, Deserialize)]
struct ScheduleStore {
    items: Vec<ScheduleItem>,
    next_id: u64,
}

// ── ScheduleWidget ────────────────────────────────────────────────────────────

pub struct ScheduleWidget {
    store: ScheduleStore,
    data_path: PathBuf,
    cursor: usize,
    /// 현재 보기 필터
    filter: ScheduleFilter,
    /// 뱃지 (미완료 오늘 일정 수)
    badge_count: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum ScheduleFilter {
    All,
    Today,
    Upcoming,
    Done,
}

impl ScheduleFilter {
    fn label(&self) -> &'static str {
        match self {
            ScheduleFilter::All => "전체",
            ScheduleFilter::Today => "오늘",
            ScheduleFilter::Upcoming => "예정",
            ScheduleFilter::Done => "완료",
        }
    }

    fn next(&self) -> Self {
        match self {
            ScheduleFilter::All => ScheduleFilter::Today,
            ScheduleFilter::Today => ScheduleFilter::Upcoming,
            ScheduleFilter::Upcoming => ScheduleFilter::Done,
            ScheduleFilter::Done => ScheduleFilter::All,
        }
    }
}

impl ScheduleWidget {
    pub fn new() -> Self {
        let data_path = data_file_path();
        let store = load_store(&data_path);
        let mut w = ScheduleWidget {
            store,
            data_path,
            cursor: 0,
            filter: ScheduleFilter::Today,
            badge_count: 0,
        };
        w.update_badge();
        w
    }

    fn update_badge(&mut self) {
        let today = Local::now().date_naive();
        self.badge_count = self
            .store
            .items
            .iter()
            .filter(|item| {
                !item.done
                    && item
                        .due_date
                        .as_ref()
                        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                        .map(|d| d <= today)
                        .unwrap_or(false)
            })
            .count();
    }

    fn filtered_items(&self) -> Vec<&ScheduleItem> {
        let today = Local::now().date_naive();
        self.store
            .items
            .iter()
            .filter(|item| match self.filter {
                ScheduleFilter::All => true,
                ScheduleFilter::Today => {
                    !item.done
                        && item
                            .due_date
                            .as_ref()
                            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                            .map(|d| d == today)
                            .unwrap_or(false)
                }
                ScheduleFilter::Upcoming => {
                    !item.done
                        && item
                            .due_date
                            .as_ref()
                            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                            .map(|d| d > today)
                            .unwrap_or(!item.done)
                }
                ScheduleFilter::Done => item.done,
            })
            .collect()
    }

    pub fn add_item(&mut self, title: String, due_date: Option<String>) {
        let id = self.store.next_id;
        self.store.next_id += 1;
        self.store.items.push(ScheduleItem {
            id,
            title,
            due_date,
            done: false,
            created_at: Local::now().format("%Y-%m-%d").to_string(),
        });
        self.save();
        self.update_badge();
    }

    pub fn toggle_item(&mut self, cursor: usize) {
        let items = self.filtered_items();
        if let Some(item) = items.get(cursor) {
            let id = item.id;
            if let Some(store_item) = self.store.items.iter_mut().find(|i| i.id == id) {
                store_item.done = !store_item.done;
            }
        }
        self.save();
        self.update_badge();
    }

    pub fn delete_item(&mut self, cursor: usize) {
        let items = self.filtered_items();
        if let Some(item) = items.get(cursor) {
            let id = item.id;
            self.store.items.retain(|i| i.id != id);
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }
        self.save();
        self.update_badge();
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.store) {
            let _ = std::fs::write(&self.data_path, json);
        }
    }

    fn build_content(&self) -> WidgetContent {
        let today = Local::now().date_naive();
        let items = self.filtered_items();

        let filter_line = format!(
            "  필터: [{}] (f로 전환)",
            self.filter.label()
        );

        let mut lines = vec![
            WidgetLine::new(filter_line).dim(),
            WidgetLine::new("─".repeat(40)).dim(),
        ];

        if items.is_empty() {
            lines.push(WidgetLine::new("  일정 없음").dim());
        } else {
            for (i, item) in items.iter().enumerate() {
                let check = if item.done { "✅" } else { "○" };
                let cursor_mark = if i == self.cursor { "▶" } else { " " };

                // 날짜 포맷 + 임박 여부
                let date_info = if let Some(date_str) = &item.due_date {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        let diff = (date - today).num_days();
                        match diff {
                            d if d < 0 => format!(" [{}일 초과]", -d),
                            0 => " [오늘]".to_string(),
                            1 => " [내일]".to_string(),
                            d if d <= 7 => format!(" [{}일 후]", d),
                            _ => format!(" [{}]", date_str),
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let line_text = format!(
                    "{} {} {}{}",
                    cursor_mark, check, item.title, date_info
                );

                let line = if item.done {
                    WidgetLine::new(line_text).dim()
                } else if let Some(date_str) = &item.due_date {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        if date < today {
                            WidgetLine::new(line_text).error_style()
                        } else if date == today {
                            WidgetLine::new(line_text).warning()
                        } else {
                            WidgetLine::new(line_text)
                        }
                    } else {
                        WidgetLine::new(line_text)
                    }
                } else {
                    WidgetLine::new(line_text)
                };

                lines.push(line);
            }
        }

        let badge = if self.badge_count > 0 {
            Some(self.badge_count.to_string())
        } else {
            None
        };

        WidgetContent {
            version: 1,
            status: WidgetStatus::Ok,
            lines,
            badge,
            actions: vec![
                WidgetActionDef {
                    key: 'j',
                    label: "다음".to_string(),
                    action: "custom".to_string(),
                    payload: Some("cursor_down".to_string()),
                },
                WidgetActionDef {
                    key: 'k',
                    label: "이전".to_string(),
                    action: "custom".to_string(),
                    payload: Some("cursor_up".to_string()),
                },
                WidgetActionDef {
                    key: ' ',
                    label: "완료 토글".to_string(),
                    action: "custom".to_string(),
                    payload: Some("toggle".to_string()),
                },
                WidgetActionDef {
                    key: 'd',
                    label: "삭제".to_string(),
                    action: "custom".to_string(),
                    payload: Some("delete".to_string()),
                },
                WidgetActionDef {
                    key: 'f',
                    label: "필터".to_string(),
                    action: "custom".to_string(),
                    payload: Some("filter".to_string()),
                },
            ],
            ..Default::default()
        }
    }
}

impl Default for ScheduleWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinWidget for ScheduleWidget {
    fn id(&self) -> &str {
        "schedule"
    }

    fn name(&self) -> &str {
        "일정"
    }

    fn tick(&mut self, _sandbox: &Sandbox) -> WidgetContent {
        self.update_badge();
        self.build_content()
    }

    fn on_key(&mut self, key: &KeyEvent) -> Option<WidgetKeyResult> {
        let items_len = self.filtered_items().len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if items_len > 0 {
                    self.cursor = (self.cursor + 1).min(items_len - 1);
                }
                Some(WidgetKeyResult::RequestRefresh)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                Some(WidgetKeyResult::RequestRefresh)
            }
            KeyCode::Char(' ') => {
                let cur = self.cursor;
                self.toggle_item(cur);
                Some(WidgetKeyResult::RequestRefresh)
            }
            KeyCode::Char('d') => {
                let cur = self.cursor;
                self.delete_item(cur);
                Some(WidgetKeyResult::RequestRefresh)
            }
            KeyCode::Char('f') => {
                self.filter = self.filter.next();
                self.cursor = 0;
                Some(WidgetKeyResult::RequestRefresh)
            }
            _ => None,
        }
    }

    fn refresh_interval_secs(&self) -> u64 {
        60 // 1분마다 뱃지 갱신
    }

    fn compact_bar(&self) -> Option<String> {
        let total = self.store.items.len();
        let done = self.store.items.iter().filter(|i| i.done).count();
        if total > 0 {
            Some(format!("📋 {}/{}", done, total))
        } else {
            None
        }
    }
}

// ── Persistence helpers ───────────────────────────────────────────────────────

fn data_file_path() -> PathBuf {
    let dir = crate::widget::discovery::widgets_dir();
    dir.join("schedule.json")
}

fn load_store(path: &PathBuf) -> ScheduleStore {
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ScheduleStore::default()
    }
}

// ── Public API for dispatcher ─────────────────────────────────────────────────

/// AppState에서 일정 위젯에 직접 접근하는 헬퍼.
pub fn find_schedule_widget(registry: &mut crate::widget::WidgetRegistry) -> Option<&mut ScheduleWidget> {
    for instance in registry.widgets.iter_mut() {
        if let crate::widget::WidgetInstance::Builtin(w) = instance {
            if w.id() == "schedule" {
                // SAFETY: dynamic downcast을 피하기 위해 별도 관리
                // schedule 위젯은 ScheduleWidget이 보장됨
                // 실제로는 trait object이므로 별도 enum variant 없이
                // dispatcher에서 ScheduleAddItem 등의 액션으로 처리
            }
        }
    }
    None
}
