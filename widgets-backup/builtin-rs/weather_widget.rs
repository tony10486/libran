// ── Built-in Weather Widget ────────────────────────────────────────────────────
//
// Open-Meteo API를 사용한 날씨 위젯 (무료, API 키 불필요).
// 위치는 config.toml의 [widgets.weather] 섹션에서 설정합니다.

use std::time::Instant;

use crossterm::event::KeyEvent;
use serde::Deserialize;

use crate::widget::sandbox::Sandbox;
use crate::widget::{BuiltinWidget, WidgetContent, WidgetKeyResult, WidgetLine, WidgetStatus};

// ── Open-Meteo API 응답 타입 ───────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct OpenMeteoResponse {
    current: Option<CurrentWeather>,
    current_units: Option<CurrentUnits>,
}

#[derive(Deserialize, Debug)]
struct CurrentWeather {
    temperature_2m: Option<f64>,
    apparent_temperature: Option<f64>,
    relative_humidity_2m: Option<f64>,
    wind_speed_10m: Option<f64>,
    weathercode: Option<i32>,
    is_day: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct CurrentUnits {
    temperature_2m: Option<String>,
    wind_speed_10m: Option<String>,
}

// ── WeatherWidget ──────────────────────────────────────────────────────────────

pub struct WeatherWidget {
    lat: f64,
    lon: f64,
    location_name: String,
    cached_content: WidgetContent,
    last_refresh: Option<Instant>,
    /// 마지막으로 수행한 fetch가 완료됐는지 (비동기 fetch 진행 중 표시용)
    is_fetching: bool,
}

impl WeatherWidget {
    pub fn new(lat: f64, lon: f64, location_name: impl Into<String>) -> Self {
        WeatherWidget {
            lat,
            lon,
            location_name: location_name.into(),
            cached_content: WidgetContent::loading(),
            last_refresh: None,
            is_fetching: false,
        }
    }

    /// 기본 위치: 서울 (설정이 없을 때)
    pub fn default_location() -> Self {
        Self::new(37.5665, 126.9780, "서울")
    }

    pub fn set_cached_content(&mut self, content: WidgetContent) {
        self.cached_content = content;
        self.last_refresh = Some(Instant::now());
        self.is_fetching = false;
    }

    pub fn needs_refresh(&self) -> bool {
        match self.last_refresh {
            None => !self.is_fetching,
            Some(t) => t.elapsed().as_secs() >= self.refresh_interval_secs() && !self.is_fetching,
        }
    }

    pub fn mark_fetching(&mut self) {
        self.is_fetching = true;
    }

    pub fn api_url(&self) -> String {
        format!(
            "https://api.open-meteo.com/v1/forecast\
            ?latitude={lat:.4}&longitude={lon:.4}\
            &current=temperature_2m,apparent_temperature,relative_humidity_2m,\
            wind_speed_10m,weathercode,is_day\
            &timezone=auto",
            lat = self.lat,
            lon = self.lon,
        )
    }

    /// HTTP 응답 JSON을 파싱하여 WidgetContent를 생성합니다.
    pub fn parse_response(&self, body: &str) -> WidgetContent {
        let resp: OpenMeteoResponse = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => return WidgetContent::error(format!("날씨 데이터 파싱 오류: {}", e)),
        };

        let current = match resp.current {
            Some(c) => c,
            None => return WidgetContent::error("날씨 데이터 없음"),
        };
        let units = resp.current_units.unwrap_or(CurrentUnits {
            temperature_2m: Some("°C".to_string()),
            wind_speed_10m: Some("km/h".to_string()),
        });

        let temp = current.temperature_2m.unwrap_or(0.0);
        let feels = current.apparent_temperature.unwrap_or(0.0);
        let humidity = current.relative_humidity_2m.unwrap_or(0.0);
        let wind = current.wind_speed_10m.unwrap_or(0.0);
        let code = current.weathercode.unwrap_or(0);
        let is_day = current.is_day.unwrap_or(1) == 1;

        let temp_unit = units.temperature_2m.as_deref().unwrap_or("°C");
        let wind_unit = units.wind_speed_10m.as_deref().unwrap_or("km/h");

        let (icon, description) = weather_code_info(code, is_day);

        let mut lines = vec![
            WidgetLine::new("").dim(),
            WidgetLine::new(format!("  📍 {}", self.location_name)).bold(),
            WidgetLine::new("").dim(),
            WidgetLine::new(format!("  {} {}  {}{}", icon, description, temp, temp_unit)).bold(),
            WidgetLine::new(format!("     체감 온도: {:.1}{}", feels, temp_unit)).dim(),
            WidgetLine::new("").dim(),
            WidgetLine::new(format!("  💧 습도: {:.0}%", humidity)),
            WidgetLine::new(format!("  🌬 풍속: {:.1} {}", wind, wind_unit)),
            WidgetLine::new("").dim(),
        ];

        // 날씨 상태에 따른 조언
        if let Some(advice) = weather_advice(code) {
            lines.push(WidgetLine::new(format!("  💡 {}", advice)).dim());
        }

        let mut content = WidgetContent {
            version: 1,
            status: WidgetStatus::Ok,
            lines,
            ..Default::default()
        };
        content.badge = Some(format!("{} {:.0}{}", icon, temp, temp_unit));
        content.actions = vec![crate::widget::WidgetActionDef {
            key: 'r',
            label: "새로고침".to_string(),
            action: "refresh".to_string(),
            payload: None,
        }];
        content
    }
}

impl BuiltinWidget for WeatherWidget {
    fn id(&self) -> &str {
        "weather"
    }

    fn name(&self) -> &str {
        "날씨"
    }

    fn tick(&mut self, _sandbox: &Sandbox) -> WidgetContent {
        // 실제 fetch는 dispatcher에서 비동기로 수행.
        // tick은 캐시된 콘텐츠만 반환.
        self.cached_content.clone()
    }

    fn on_key(&mut self, key: &KeyEvent) -> Option<WidgetKeyResult> {
        use crossterm::event::KeyCode;
        if let KeyCode::Char('r') = key.code {
            return Some(WidgetKeyResult::RequestRefresh);
        }
        None
    }

    fn refresh_interval_secs(&self) -> u64 {
        1800 // 30분
    }

    fn compact_bar(&self) -> Option<String> {
        self.cached_content.badge.clone()
    }

    fn pending_fetch(&mut self) -> Option<String> {
        if self.needs_refresh() {
            self.is_fetching = true;
            Some(self.api_url())
        } else {
            None
        }
    }

    fn on_fetch_complete(&mut self, body: &str) -> WidgetContent {
        let content = self.parse_response(body);
        self.cached_content = content.clone();
        self.last_refresh = Some(Instant::now());
        self.is_fetching = false;
        content
    }
}

// ── WMO 날씨 코드 → 아이콘 + 설명 ────────────────────────────────────────────

fn weather_code_info(code: i32, is_day: bool) -> (&'static str, &'static str) {
    match code {
        0 => if is_day { ("☀️", "맑음") } else { ("🌙", "맑음") },
        1 => if is_day { ("🌤", "대체로 맑음") } else { ("🌤", "대체로 맑음") },
        2 => ("⛅", "부분 흐림"),
        3 => ("☁️", "흐림"),
        45 | 48 => ("🌫", "안개"),
        51 | 53 | 55 => ("🌦", "이슬비"),
        61 | 63 | 65 => ("🌧", "비"),
        71 | 73 | 75 => ("🌨", "눈"),
        77 => ("❄️", "눈보라"),
        80 | 81 | 82 => ("🌦", "소나기"),
        85 | 86 => ("🌨", "눈 소나기"),
        95 => ("⛈", "뇌우"),
        96 | 99 => ("⛈", "우박 뇌우"),
        _ => ("🌡", "알 수 없음"),
    }
}

fn weather_advice(code: i32) -> Option<&'static str> {
    match code {
        61..=65 | 80..=82 => Some("우산을 챙기세요"),
        71..=77 | 85..=86 => Some("방한 용품을 챙기세요"),
        95..=99 => Some("외출을 자제하세요"),
        45 | 48 => Some("운전 시 주의하세요"),
        _ => None,
    }
}
