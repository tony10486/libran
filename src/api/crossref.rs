use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub fn create_polite_http_client(user_email: Option<&str>) -> Result<Client> {
    let mut headers = reqwest::header::HeaderMap::new();

    let agent_string = if let Some(email) = user_email {
        format!("Libran/0.1 (mailto:{})", email)
    } else {
        "Libran/0.1".to_string()
    };

    if let Ok(value) = reqwest::header::HeaderValue::from_str(&agent_string) {
        headers.insert(reqwest::header::USER_AGENT, value);
    }

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(3)
        .build()?;

    Ok(client)
}

pub async fn fetch_by_doi(client: &Client, doi: &str) -> Result<String> {
    let url = format!("https://api.crossref.org/works/{}", doi);
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;
    Ok(body)
}

pub async fn search_by_title(client: &Client, title: &str) -> Result<String> {
    let url = format!(
        "https://api.crossref.org/works?query.bibliographic={}&rows=5",
        urlencoding::encode(title)
    );
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;
    Ok(body)
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u32)
                }
            })
            .collect()
    }
}
