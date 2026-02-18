use anyhow::{anyhow, Result};
use std::sync::LazyLock;
use regex::Regex;

/// scrapped data from Discord's live /app page
#[derive(Debug, Clone)]
pub struct LiveBuildData {
    pub build_hash: String,
    pub channel: String,
    pub scripts: Vec<String>,
    pub global_env: serde_json::Value,
    pub timestamp: i64,
}

static SCRIPT_SRC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<script[^>]+src="(/assets/[^"]+\.js)"[^>]*>"#).unwrap());

static CSS_HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<link[^>]+href="(/assets/[^"]+\.css)"[^>]+rel="stylesheet""#).unwrap());

static GLOBAL_ENV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"window\.GLOBAL_ENV\s*=\s*(\{.+?\})\s*</script>"#).unwrap());

/// fetches the live Discord client page and extracts build info.
pub async fn fetch_live_build(client: &reqwest::Client, base_url: &str) -> Result<LiveBuildData> {
    let url = format!("{}/app", base_url.trim_end_matches('/'));

    let resp = client
        .get(&url)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Discord returned {}", resp.status()));
    }

    let html = resp.text().await?;
    parse_discord_html(&html)
}

pub fn parse_discord_html(html: &str) -> Result<LiveBuildData> {
    // extract GLOBAL_ENV JSON
    let global_env: serde_json::Value = GLOBAL_ENV_RE
        .captures(html)
        .and_then(|cap| {
            let raw = cap[1].replace("Date.now()", "0");
            serde_json::from_str(&raw).ok()
        })
        .unwrap_or(serde_json::json!({}));

    // extract build hash from VERSION_HASH or SENTRY_TAGS.buildId
    let build_hash = global_env
        .get("VERSION_HASH")
        .or_else(|| global_env.pointer("/SENTRY_TAGS/buildId"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Could not find build hash in GLOBAL_ENV"))?
        .to_string();

    let channel = global_env
        .get("RELEASE_CHANNEL")
        .and_then(|v| v.as_str())
        .unwrap_or("canary")
        .to_string();

    let timestamp = global_env
        .get("BUILT_AT")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    // extract script srcs from <script defer src="/assets/...">
    let mut scripts: Vec<String> = SCRIPT_SRC_RE
        .captures_iter(html)
        .map(|cap| cap[1].trim_start_matches("/assets/").to_string())
        .collect();

    // also grab CSS files as assets to download
    for cap in CSS_HREF_RE.captures_iter(html) {
        scripts.push(cap[1].trim_start_matches("/assets/").to_string());
    }

    if scripts.is_empty() {
        return Err(anyhow!("No scripts found in Discord HTML"));
    }

    tracing::info!(
        "Scraped live build: hash={}, channel={}, {} scripts",
        build_hash, channel, scripts.len()
    );

    Ok(LiveBuildData {
        build_hash,
        channel,
        scripts,
        global_env,
        timestamp,
    })
}
