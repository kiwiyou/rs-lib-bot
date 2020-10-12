use chrono::{DateTime, Utc};
use reqwest::Client;

#[derive(serde::Deserialize)]
pub struct CrateInfo {
    pub version: String,
    pub description: Option<String>,
    pub downloads: usize,
    pub license: Option<String>,
    pub crate_size: Option<usize>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub dependencies: usize,
    pub dev_dependencies: usize,
    pub build_dependencies: usize,
}

// Use kiwiyou/crate-search-cache due to crate.io ratelimit
pub async fn search_crate(
    client: &Client,
    search_url: &str,
    name: &str,
) -> anyhow::Result<Option<CrateInfo>> {
    let url = format!("{}/{}", search_url, name);
    let response = client.get(&url).send().await?;
    if response.status().is_client_error() {
        Ok(None)
    } else {
        let result = response.json().await?;
        Ok(Some(result))
    }
}
