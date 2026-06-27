use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsBackend {
    SemanticScholar,
    OpenAlex,
}

impl MetricsBackend {
    pub fn as_str(&self) -> &str {
        match self {
            MetricsBackend::SemanticScholar => "semantic_scholar",
            MetricsBackend::OpenAlex => "openalex",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            MetricsBackend::SemanticScholar => "Semantic Scholar",
            MetricsBackend::OpenAlex => "OpenAlex",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "openalex" => MetricsBackend::OpenAlex,
            _ => MetricsBackend::SemanticScholar,
        }
    }

    pub fn requires_api_key(&self) -> bool {
        matches!(self, MetricsBackend::OpenAlex)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorMetrics {
    pub name: String,
    pub h_index: Option<i64>,
    pub i10_index: Option<i64>,
    pub works_count: Option<i64>,
    pub cited_by_count: Option<i64>,
    pub source: MetricsBackend,
}

pub fn cache_key(backend: MetricsBackend, author_name: &str) -> String {
    format!(
        "metrics:{}:{}",
        backend.as_str(),
        author_name.to_lowercase()
    )
}

pub fn get_cached_metrics(
    conn: &Connection,
    backend: MetricsBackend,
    author_name: &str,
    max_age_days: u32,
) -> Result<Option<AuthorMetrics>> {
    let key = cache_key(backend, author_name);
    let now = Utc::now();
    let max_age = Duration::days(max_age_days as i64);
    let threshold = now - max_age;

    let result = conn.query_row(
        "SELECT response_json, fetched_at FROM api_cache WHERE cache_key = ?1",
        params![&key],
        |row| {
            let json: String = row.get(0)?;
            let fetched_at: String = row.get(1)?;
            Ok((json, fetched_at))
        },
    );

    match result {
        Ok((json, fetched_at_str)) => {
            if let Ok(fetched_at) = chrono::DateTime::parse_from_rfc3339(&fetched_at_str) {
                if fetched_at.with_timezone(&Utc) > threshold {
                    let metrics: AuthorMetrics = serde_json::from_str(&json)?;
                    return Ok(Some(metrics));
                }
            }
            Ok(None)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_stale_cached_metrics(
    conn: &Connection,
    backend: MetricsBackend,
    author_name: &str,
) -> Result<Option<AuthorMetrics>> {
    let key = cache_key(backend, author_name);
    let result = conn.query_row(
        "SELECT response_json FROM api_cache WHERE cache_key = ?1",
        params![&key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(json) => {
            let metrics: AuthorMetrics = serde_json::from_str(&json)?;
            Ok(Some(metrics))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn store_cached_metrics(
    conn: &Connection,
    backend: MetricsBackend,
    author_name: &str,
    metrics: &AuthorMetrics,
) -> Result<()> {
    let key = cache_key(backend, author_name);
    let json = serde_json::to_string(metrics)?;
    let now = Utc::now();
    let expires = now + Duration::days(365);
    conn.execute(
        "INSERT OR REPLACE INTO api_cache (cache_key, source, response_json, fetched_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&key, backend.as_str(), &json, now.to_rfc3339(), expires.to_rfc3339()],
    )?;
    Ok(())
}

pub async fn fetch_author_metrics(
    backend: MetricsBackend,
    api_key: Option<&str>,
    author_name: &str,
) -> Result<AuthorMetrics> {
    const MAX_RETRIES: u32 = 3;
    let mut backoff_secs: u64 = 1;
    let mut last_err: Option<anyhow::Error> = None;

    for attempt in 0..=MAX_RETRIES {
        let result = match backend {
            MetricsBackend::SemanticScholar => fetch_from_semantic_scholar(author_name).await,
            MetricsBackend::OpenAlex => fetch_from_openalex(api_key, author_name).await,
        };

        match result {
            Ok(m) => return Ok(m),
            Err(e) => {
                let is_rate_limited = e.to_string().contains("429");
                if !is_rate_limited || attempt == MAX_RETRIES {
                    return Err(e);
                }
                last_err = Some(e);
                tokio::time::sleep(StdDuration::from_secs(backoff_secs)).await;
                backoff_secs = backoff_secs.saturating_mul(2);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("재시도 실패")))
}

async fn fetch_from_semantic_scholar(author_name: &str) -> Result<AuthorMetrics> {
    let client = crate::api::http_client::create_polite_http_client(None, 15)?;
    let url = format!(
        "https://api.semanticscholar.org/graph/v1/author/search?query={}&fields=name,paperCount,citationCount,hIndex&limit=1",
        urlencoding::encode(author_name)
    );

    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;

    if status.as_u16() == 429 {
        anyhow::bail!(
            "Semantic Scholar 요청 제한 초과 (429). 잠시 후 재시도하거나 OpenAlex(K 키)를 사용하세요."
        );
    }
    if !status.is_success() {
        anyhow::bail!("Semantic Scholar 오류: HTTP {}", status);
    }

    let resp_data: S2SearchResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Semantic Scholar 응답 파싱 실패: {e}"))?;

    let author = resp_data
        .data
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("검색 결과 없음: {}", author_name))?;

    Ok(AuthorMetrics {
        name: author.name,
        h_index: author.h_index,
        i10_index: None,
        works_count: author.paper_count,
        cited_by_count: author.citation_count,
        source: MetricsBackend::SemanticScholar,
    })
}

async fn fetch_from_openalex(api_key: Option<&str>, author_name: &str) -> Result<AuthorMetrics> {
    let client = crate::api::http_client::create_polite_http_client(None, 15)?;
    let mut url = format!(
        "https://api.openalex.org/authors?search={}&per_page=1&select=id,display_name,works_count,cited_by_count,summary_stats",
        urlencoding::encode(author_name)
    );

    if let Some(key) = api_key {
        if !key.is_empty() {
            url.push_str(&format!("&api_key={}", urlencoding::encode(key)));
        }
    }

    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        anyhow::bail!("OpenAlex 오류: HTTP {}", status);
    }

    let resp_data: OpenAlexListResponse =
        serde_json::from_str(&body).map_err(|e| anyhow::anyhow!("OpenAlex 응답 파싱 실패: {e}"))?;

    let author = resp_data
        .results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("검색 결과 없음: {}", author_name))?;

    Ok(AuthorMetrics {
        name: author.display_name,
        h_index: author.summary_stats.as_ref().and_then(|s| s.h_index),
        i10_index: author.summary_stats.as_ref().and_then(|s| s.i10_index),
        works_count: Some(author.works_count),
        cited_by_count: Some(author.cited_by_count),
        source: MetricsBackend::OpenAlex,
    })
}

#[derive(Deserialize)]
struct S2SearchResponse {
    data: Vec<S2Author>,
}

#[derive(Deserialize)]
struct S2Author {
    name: String,
    #[serde(default)]
    paper_count: Option<i64>,
    #[serde(default)]
    citation_count: Option<i64>,
    #[serde(default)]
    h_index: Option<i64>,
}

#[derive(Deserialize)]
struct OpenAlexListResponse {
    results: Vec<OpenAlexAuthor>,
}

#[derive(Deserialize)]
struct OpenAlexAuthor {
    display_name: String,
    works_count: i64,
    cited_by_count: i64,
    #[serde(default)]
    summary_stats: Option<OpenAlexSummaryStats>,
}

#[derive(Deserialize)]
struct OpenAlexSummaryStats {
    #[serde(rename = "h_index")]
    h_index: Option<i64>,
    #[serde(rename = "i10_index")]
    i10_index: Option<i64>,
}
