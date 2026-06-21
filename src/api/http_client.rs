//! 공유 HTTP 클라이언트 생성기.
//!
//! `documentation.md` §6.2 Polite Pool 정책에 따라 모든 외부 API 요청에
//! `User-Agent` 헤더를 포함합니다. 이 모듈을 통해 crossref·metrics 등
//! 모든 API 호출이 동일한 Polite Pool 정책을 따릅니다.

use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub fn create_polite_http_client(user_email: Option<&str>, timeout_secs: u64) -> Result<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    let agent_string = match user_email {
        Some(email) if !email.is_empty() => format!("Libran/0.1 (mailto:{})", email),
        _ => "Libran/0.1".to_string(),
    };

    if let Ok(value) = reqwest::header::HeaderValue::from_str(&agent_string) {
        headers.insert(reqwest::header::USER_AGENT, value);
    }

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(timeout_secs))
        .pool_max_idle_per_host(3)
        .build()?;

    Ok(client)
}
