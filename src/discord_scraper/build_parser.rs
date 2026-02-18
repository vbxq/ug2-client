use super::types::*;
use anyhow::Result;

pub fn parse_build(data: &BuildData) -> Result<BuildInfo> {
    // extract channel from GLOBAL_ENV if present, default to "canary"
    let channel = data.global_env
        .get("RELEASE_CHANNEL")
        .and_then(|v| v.as_str())
        .unwrap_or("canary")
        .to_string();

    // extract timestamp: prefer date field (RFC3339), fallback to GLOBAL_ENV.HTML_TIMESTAMP
    let timestamp = chrono::DateTime::parse_from_rfc3339(&data.date)
        .map(|dt| dt.timestamp_millis())
        .unwrap_or_else(|_| {
            data.global_env
                .get("HTML_TIMESTAMP")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
        });

    let scripts: Vec<String> = data.scripts.iter()
        .map(|s| s.trim_start_matches("/assets/").to_string())
        .collect();

    Ok(BuildInfo {
        build_hash: data.id.clone(),
        channel,
        scripts,
        index_scripts: Vec::new(),
        timestamp,
        global_env: data.global_env.clone(),
    })
}
