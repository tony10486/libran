use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub fn create_client() -> Result<Client> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    Ok(client)
}

pub async fn fetch_by_arxiv_id(client: &Client, arxiv_id: &str) -> Result<String> {
    let url = format!("http://export.arxiv.org/api/query?id_list={}", arxiv_id);
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;
    Ok(body)
}
