use super::extractor;
use anyhow::Result;
use bytes::Bytes;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

const MAX_CONCURRENT: usize = 100;
const MAX_RETRIES: u32 = 3;

pub struct AssetDownloader {
    client: Client,
    cache_path: PathBuf,
    base_url: String,
    semaphore: Arc<Semaphore>,
}

impl AssetDownloader {
    pub fn new(cache_path: PathBuf, base_url: &str) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(50)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            cache_path,
            base_url: base_url.to_string(),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT)),
        }
    }

    pub async fn download_build(&self, build_hash: &str, initial_scripts: &[String]) -> Result<Vec<String>> {
        let build_dir = self.cache_path.join(build_hash);
        tokio::fs::create_dir_all(&build_dir).await?;

        let mut known_assets: HashSet<String> = HashSet::new();
        let mut queue: Vec<String> = initial_scripts.to_vec();
        let mut downloaded: Vec<String> = Vec::new();

        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed}] Downloaded: {pos} | Queue: {msg}")
            .unwrap());

        while !queue.is_empty() {
            pb.set_message(format!("{}", queue.len()));

            let batch: Vec<String> = queue.drain(..).collect();
            let mut handles = Vec::new();

            for asset_name in batch {
                let asset_name = if asset_name.contains('.') {
                    asset_name
                } else {
                    format!("{}.js", asset_name)
                };

                if !known_assets.insert(asset_name.clone()) {
                    continue;
                }

                let client = self.client.clone();
                let base_url = self.base_url.clone();
                let build_dir = build_dir.clone();
                let sem = self.semaphore.clone();

                handles.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    let result = download_single_asset(&client, &base_url, &asset_name, &build_dir).await;
                    (asset_name, result)
                }));
            }

            for handle in handles {
                let (asset_name, result) = handle.await?;
                match result {
                    Ok(new_refs) => {
                        downloaded.push(asset_name);
                        pb.inc(1);
                        for r in new_refs {
                            if !known_assets.contains(&r) {
                                queue.push(r);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to download {}: {}", asset_name, e);
                    }
                }
            }
        }

        pb.finish_with_message(format!("Done! {} assets downloaded", downloaded.len()));
        Ok(downloaded)
    }

}

async fn download_single_asset(
    client: &Client,
    base_url: &str,
    asset_name: &str,
    build_dir: &Path,
) -> Result<Vec<String>> {
    let url = format!("{}/assets/{}", base_url, asset_name);
    let dest = build_dir.join(asset_name);

    if dest.exists() {
        if asset_name.ends_with(".js") || asset_name.ends_with(".css") {
            let content = tokio::fs::read_to_string(&dest).await?;
            return Ok(extractor::extract_asset_refs(&content).into_iter().collect());
        }
        return Ok(vec![]);
    }

    let bytes = download_with_retry(client, &url, MAX_RETRIES).await?;

    let is_text = asset_name.ends_with(".js") || asset_name.ends_with(".css");
    if !is_text {
        tokio::fs::write(&dest, &bytes).await?;
        return Ok(vec![]);
    }

    let content = String::from_utf8_lossy(&bytes);
    let refs: Vec<String> = extractor::extract_asset_refs(&content).into_iter().collect();
    tokio::fs::write(&dest, bytes).await?;

    Ok(refs)
}

async fn download_with_retry(client: &Client, url: &str, max_retries: u32) -> Result<Bytes> {
    let mut last_err = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        match client.get(url).send().await {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::NOT_FOUND {
                    anyhow::bail!("404 Not Found: {}", url);
                }
                let resp = resp.error_for_status()?;
                return Ok(resp.bytes().await?);
            }
            Err(e) => {
                tracing::debug!("Attempt {} failed for {}: {}", attempt + 1, url, e);
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap().into())
}
