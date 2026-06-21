use anyhow::Result;
use reqwest::Client;

pub fn create_polite_http_client(user_email: Option<&str>) -> Result<Client> {
    crate::api::http_client::create_polite_http_client(user_email, 10)
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
