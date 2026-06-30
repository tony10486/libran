// ── Widget Sandbox ─────────────────────────────────────────────────────────────
//
// 보안 정책을 강제하는 게이트웨이.
// 모든 위젯의 HTTP 요청은 이 모듈을 통해야 합니다.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};

// ── Policy ────────────────────────────────────────────────────────────────────

/// 위젯 한 개에 적용되는 샌드박스 정책.
#[derive(Clone, Debug)]
pub struct SandboxPolicy {
    /// 허용된 도메인 목록 (정확한 호스트명 또는 "*" 와일드카드 불허)
    pub allowed_domains: Vec<String>,
    /// 분당 최대 HTTP 요청 수
    pub rate_limit_per_minute: u32,
    /// 단일 HTTP 응답 최대 바이트
    pub max_response_bytes: usize,
    /// HTTP 요청 타임아웃 (초)
    pub request_timeout_secs: u64,
    /// HTTPS 전용 강제 여부
    pub https_only: bool,
    /// 허용된 HTTP 메서드 (소문자)
    pub allowed_methods: Vec<String>,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        SandboxPolicy {
            allowed_domains: Vec::new(),
            rate_limit_per_minute: 30,
            max_response_bytes: 1_048_576, // 1 MB
            request_timeout_secs: 10,
            https_only: true,
            allowed_methods: vec!["get".to_string()],
        }
    }
}

// ── Rate Limiter (Token Bucket) ───────────────────────────────────────────────

/// 도메인별 요청 빈도 추적기.
#[derive(Debug)]
struct RateLimiter {
    /// 도메인 → (마지막 요청들의 타임스탬프 deque)
    buckets: HashMap<String, Vec<Instant>>,
    limit_per_minute: u32,
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        RateLimiter {
            buckets: HashMap::new(),
            limit_per_minute,
        }
    }

    /// 요청 허용 여부 확인. 허용되면 버킷에 기록하고 true 반환.
    fn check_and_record(&mut self, domain: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let bucket = self.buckets.entry(domain.to_string()).or_default();

        // 1분 이상 지난 항목 제거
        bucket.retain(|&ts| now.duration_since(ts) < window);

        if bucket.len() >= self.limit_per_minute as usize {
            return false;
        }
        bucket.push(now);
        true
    }
}

// ── Sandbox ───────────────────────────────────────────────────────────────────

/// 위젯 API 호출 샌드박스.
/// Clone해서 여러 위젯이 공유할 수 있도록 Arc<Mutex<>> 내부 상태 사용.
#[derive(Clone)]
pub struct Sandbox {
    inner: Arc<Mutex<SandboxInner>>,
    pub policy: SandboxPolicy,
}

struct SandboxInner {
    rate_limiter: RateLimiter,
    http_client: reqwest::Client,
}

impl Sandbox {
    pub fn new(policy: SandboxPolicy) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(policy.request_timeout_secs))
            .user_agent(concat!("Libran-Widget/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();

        Sandbox {
            inner: Arc::new(Mutex::new(SandboxInner {
                rate_limiter: RateLimiter::new(policy.rate_limit_per_minute),
                http_client: client,
            })),
            policy,
        }
    }

    /// 샌드박스를 통해 HTTP GET 요청을 수행합니다.
    /// 도메인 허용목록, HTTPS, 빈도 제한, 응답 크기 제한을 모두 검증합니다.
    pub async fn http_get(
        &self,
        url: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<String> {
        self.validate_url(url)?;
        let host = extract_host(url)?;
        self.check_rate_limit(&host)?;

        // Clone the client and drop the lock before async operations.
        // std::sync::MutexGuard is not Send, so holding it across .await
        // would make the future non-Send (unusable with tokio::spawn).
        let client = {
            let inner = self.inner.lock().map_err(|_| anyhow::anyhow!("lock error"))?;
            inner.http_client.clone()
        };

        let mut builder = client.get(url);

        if let Some(hdrs) = headers {
            for (k, v) in hdrs {
                // Authorization, Cookie 등 민감 헤더 차단
                let key_lower = k.to_lowercase();
                if key_lower == "authorization" || key_lower == "cookie" || key_lower == "set-cookie" {
                    bail!("차단된 헤더: {}", k);
                }
                builder = builder.header(k.as_str(), v.as_str());
            }
        }

        let resp = builder.send().await?;
        let status = resp.status();

        // 응답 크기 제한: content-length 헤더로 먼저 체크
        if let Some(len) = resp.content_length() {
            if len as usize > self.policy.max_response_bytes {
                bail!(
                    "응답 크기 초과: {} bytes (최대 {})",
                    len,
                    self.policy.max_response_bytes
                );
            }
        }

        let bytes = resp.bytes().await?;
        if bytes.len() > self.policy.max_response_bytes {
            bail!(
                "응답 크기 초과: {} bytes (최대 {})",
                bytes.len(),
                self.policy.max_response_bytes
            );
        }

        if !status.is_success() {
            bail!("HTTP 오류: {}", status);
        }

        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    // ── Internal validators ────────────────────────────────────────────────────

    fn validate_url(&self, url: &str) -> Result<()> {
        // HTTPS 강제
        if self.policy.https_only && !url.starts_with("https://") {
            bail!("HTTPS만 허용됩니다: {}", url);
        }

        // 도메인 허용목록 검사
        let host = extract_host(url)?;
        let allowed = self.policy.allowed_domains.iter().any(|d| {
            // 서브도메인 포함 매칭: ".example.com" 패턴 지원
            if let Some(suffix) = d.strip_prefix('*') {
                host.ends_with(suffix)
            } else {
                host == *d || host.ends_with(&format!(".{}", d))
            }
        });

        if !allowed {
            bail!(
                "도메인이 허용 목록에 없습니다: {} (허용: {:?})",
                host,
                self.policy.allowed_domains
            );
        }

        Ok(())
    }

    fn check_rate_limit(&self, host: &str) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|_| anyhow::anyhow!("lock error"))?;
        if !inner.rate_limiter.check_and_record(host) {
            bail!(
                "요청 빈도 제한 초과: {} (분당 최대 {}회)",
                host,
                self.policy.rate_limit_per_minute
            );
        }
        Ok(())
    }
}

/// URL에서 호스트명 추출.
fn extract_host(url: &str) -> Result<String> {
    // 간단한 파싱: https://host/path 에서 host 추출
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| anyhow::anyhow!("잘못된 URL 형식: {}", url))?;

    let host = without_scheme.split('/').next().unwrap_or("");
    // 포트 번호 제거
    let host = host.split(':').next().unwrap_or("");
    if host.is_empty() {
        bail!("URL에서 호스트를 추출할 수 없습니다: {}", url);
    }
    Ok(host.to_lowercase())
}

// ── Default global sandbox ────────────────────────────────────────────────────

/// 앱 시작 시 사용할 기본 샌드박스 정책.
/// config.toml의 [widgets.sandbox] 값으로 오버라이드됩니다.
pub fn default_sandbox() -> Sandbox {
    Sandbox::new(SandboxPolicy {
        // 기본 내장 위젯 도메인
        allowed_domains: vec![
            "api.open-meteo.com".to_string(),
        ],
        rate_limit_per_minute: 30,
        max_response_bytes: 1_048_576,
        request_timeout_secs: 10,
        https_only: true,
        allowed_methods: vec!["get".to_string()],
    })
}

/// 위젯의 허용 도메인 목록을 글로벌 정책에 합산하여 위젯 전용 샌드박스를 생성.
pub fn make_widget_sandbox(global: &Sandbox, widget_domains: &[String]) -> Sandbox {
    let mut policy = global.policy.clone();
    for d in widget_domains {
        if !policy.allowed_domains.contains(d) {
            policy.allowed_domains.push(d.clone());
        }
    }
    Sandbox::new(policy)
}
