// ── Built-in Clock Widget ──────────────────────────────────────────────────────

use chrono::Local;
use crossterm::event::KeyEvent;

use crate::widget::sandbox::Sandbox;
use crate::widget::{BuiltinWidget, WidgetContent, WidgetKeyResult, WidgetLine};

pub struct ClockWidget {
    show_seconds: bool,
}

impl ClockWidget {
    pub fn new() -> Self {
        ClockWidget { show_seconds: true }
    }
}

impl Default for ClockWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinWidget for ClockWidget {
    fn id(&self) -> &str {
        "clock"
    }

    fn name(&self) -> &str {
        "시계"
    }

    fn tick(&mut self, _sandbox: &Sandbox) -> WidgetContent {
        let now = Local::now();

        let time_str = if self.show_seconds {
            now.format("%H:%M:%S").to_string()
        } else {
            now.format("%H:%M").to_string()
        };

        let date_str = now.format("%Y년 %m월 %d일").to_string();

        let weekday_str = now.format("%a").to_string();
        let weekday = match weekday_str.as_str() {
            "Mon" => "월요일",
            "Tue" => "화요일",
            "Wed" => "수요일",
            "Thu" => "목요일",
            "Fri" => "금요일",
            "Sat" => "토요일",
            "Sun" => "일요일",
            _ => weekday_str.as_str(),
        };

        let lines = vec![
            WidgetLine::new("").dim(),
            WidgetLine::new(format!("  🕐  {}", time_str)).bold().center(),
            WidgetLine::new("").dim(),
            WidgetLine::new(format!("  {}  {}", date_str, weekday)).center(),
            WidgetLine::new("").dim(),
        ];

        let mut content = WidgetContent::simple(lines);
        content.actions = vec![crate::widget::WidgetActionDef {
            key: 's',
            label: if self.show_seconds { "초 숨기기".to_string() } else { "초 표시".to_string() },
            action: "custom".to_string(),
            payload: Some("toggle_seconds".to_string()),
        }];
        content
    }

    fn on_key(&mut self, key: &KeyEvent) -> Option<WidgetKeyResult> {
        use crossterm::event::KeyCode;
        if let KeyCode::Char('s') = key.code {
            self.show_seconds = !self.show_seconds;
            return Some(WidgetKeyResult::RequestRefresh);
        }
        None
    }

    fn refresh_interval_secs(&self) -> u64 {
        1 // 매초 갱신
    }

    fn compact_bar(&self) -> Option<String> {
        let now = Local::now();
        let h = now.format("%I:%M %p").to_string();
        Some(h)
    }
}
