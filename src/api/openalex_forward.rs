use anyhow::{Result, anyhow};
use serde::Deserialize;

const OPENALEX_BASE: &str = "https://api.openalex.org";

#[derive(Clone, Debug, Default)]
pub struct ForwardCitation {
    pub title: String,
    pub year: Option<i64>,
    pub doi: Option<String>,
    pub authors: Vec<String>,
}

/// Fetch forward citations for a given DOI via OpenAlex.
/// Returns the list of works that cite the given DOI.
pub async fn fetch_forward_citations(doi: &str) -> Result<(Vec<ForwardCitation>, i64)> {
    let client = reqwest::Client::builder()
        .user_agent("libran/0.1 (mailto:libran@example.com)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Step 1: resolve the work ID from DOI
    let work_url = format!("{}/works/doi:{}", OPENALEX_BASE, doi);
    let resp = client.get(&work_url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("OpenAlex work 조회 실패: HTTP {}", resp.status()));
    }
    let work_json: serde_json::Value = resp.json().await?;
    let work_id = work_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("OpenAlex 응답에 id 없음"))?;

    // Step 2: fetch works that cite this work
    let cited_count = work_json
        .get("cited_by_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let citations_url = format!(
        "{}/works?filter=cites:{}&per-page=200&select=id,title,publication_year,doi,authorships",
        OPENALEX_BASE, work_id
    );
    let resp = client.get(&citations_url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "OpenAlex 전방 인용 조회 실패: HTTP {}",
            resp.status()
        ));
    }

    let citations_json: OpenAlexResponse = resp.json().await?;
    let citations = citations_json
        .results
        .into_iter()
        .map(|r| ForwardCitation {
            title: r.title.unwrap_or_default(),
            year: r.publication_year,
            doi: r.doi,
            authors: r
                .authorships
                .into_iter()
                .filter_map(|a| {
                    a.author.and_then(|au| {
                        au.display_name
                            .map(|name| if name.is_empty() { name } else { name })
                    })
                })
                .collect(),
        })
        .collect();

    Ok((citations, cited_count))
}

#[derive(Deserialize)]
struct OpenAlexResponse {
    results: Vec<OpenAlexWork>,
}

#[derive(Deserialize)]
struct OpenAlexWork {
    title: Option<String>,
    publication_year: Option<i64>,
    doi: Option<String>,
    authorships: Vec<Authorship>,
}

#[derive(Deserialize)]
struct Authorship {
    author: Option<OpenAlexAuthor>,
}

#[derive(Deserialize)]
struct OpenAlexAuthor {
    display_name: Option<String>,
}
