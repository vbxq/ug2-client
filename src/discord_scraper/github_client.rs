use super::types::*;
use anyhow::{Context, Result};
use reqwest::Client;

pub struct GitHubClient {
    client: Client,
    raw_base: String,
}

// honestly, we don't even need this for now
// maybe we can use this https://github.com/xhyrom/discord-datamining
impl GitHubClient {
    pub fn new(repo: &str) -> Self {
        Self {
            client: Client::builder()
                .user_agent("ug2-client/0.1")
                .build()
                .expect("Failed to build HTTP client"),
            raw_base: format!("https://raw.githubusercontent.com/{}/main", repo),
        }
    }

    pub async fn fetch_build_index(&self) -> Result<BuildIndex> {
        let url = format!("{}/builds.json", self.raw_base);
        tracing::info!("Fetching build index from {}", url);
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let index: BuildIndex = resp.json().await?;
        tracing::info!("Fetched {} builds from index", index.len());
        Ok(index)
    }

    pub async fn fetch_build_by_hash(&self, hash: &str) -> Result<BuildData> {
        let index = self.fetch_build_index().await?;
        let entry = index.get(hash)
            .context(format!("Build hash {} not found in index", hash))?;
        self.fetch_build_at_path(&entry.path, hash).await
    }

    pub async fn fetch_build_at_path(&self, path: &str, hash: &str) -> Result<BuildData> {
        let url = format!("{}/{}/{}.json", self.raw_base, path, hash);
        tracing::info!("Fetching build data from {}", url);
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let build: BuildData = resp.json().await?;
        Ok(build)
    }

    pub async fn fetch_latest_build(&self) -> Result<(String, BuildIndexEntry)> {
        let index = self.fetch_build_index().await?;
        let (hash, entry) = index.iter()
            .max_by_key(|(_, e)| e.date)
            .context("No builds found in index")?;
        Ok((hash.clone(), entry.clone()))
    }

}
