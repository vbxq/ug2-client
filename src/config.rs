use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub bind_addr: String,
    pub api_base_url: String,
    pub discord_base_url: String,
    pub asset_base_url: String,
    pub github_builds_repo: String,
    pub cache_path: PathBuf,
    pub patch_config: PatchConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchConfig {
    pub patches: PatchToggles,
    pub branding: BrandingConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub trust_proxy_headers: bool,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            trust_proxy_headers: false,
            rate_limit_enabled: false,
            rate_limit_requests: 60,
            rate_limit_window_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchToggles {
    pub nitro_rebranding: bool,
    pub discord_rebranding: bool,
    pub title_rebranding: bool,
    pub server_to_guild: bool,
    pub sentry_redirect: bool,
    pub status_page_redirect: bool,
    pub prevent_localstorage_deletion: bool,
    pub fast_identify: bool,
    pub gateway_reconnect: bool,
    pub remove_qr_login: bool,
    pub enable_dev_experiments: bool,
    pub remove_modals: bool,
    pub no_xss_warning: bool,
    pub vencord: bool,
    pub api_proxy: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrandingConfig {
    pub instance_name: String,
    pub instance_url: String,
    pub sentry_url: String,
    pub status_url: String,
    pub gateway_url: Option<String>,
    pub cdn_url: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let patch_config_str = std::fs::read_to_string("patch_config.toml")?;
        let patch_config: PatchConfig = toml::from_str(&patch_config_str)?;
        let resolved = ResolvedUrls::from_config(
            &patch_config.branding,
            std::env::var("DISCORD_BASE_URL").ok(),
            std::env::var("DISCORD_UPSTREAM_BASE_URL").ok(),
            std::env::var("DISCORD_ASSET_BASE_URL").ok(),
        );

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            api_base_url: resolved.api_base_url,
            discord_base_url: resolved.discord_base_url,
            asset_base_url: resolved.asset_base_url,
            github_builds_repo: std::env::var("GITHUB_BUILDS_REPO").unwrap_or_else(|_| "Discord-Build-Logger/Builds".into()),
            cache_path: PathBuf::from(std::env::var("CACHE_PATH").unwrap_or_else(|_| "./assets/cache".into())),
            patch_config,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedUrls {
    api_base_url: String,
    discord_base_url: String,
    asset_base_url: String,
}

impl ResolvedUrls {
    fn from_config(
        branding: &BrandingConfig,
        legacy_discord_base_url: Option<String>,
        upstream_override: Option<String>,
        asset_override: Option<String>,
    ) -> Self {
        let api_base_url = normalize_base_url(&branding.instance_url);

        let discord_base_url = upstream_override
            .or(legacy_discord_base_url.filter(|url| normalize_base_url(url) != api_base_url))
            .map(|url| normalize_base_url(&url))
            .unwrap_or_else(|| "https://canary.discord.com".into());

        let asset_base_url = asset_override
            .map(|url| normalize_base_url(&url))
            .unwrap_or_else(|| "https://discord.com".into());

        Self {
            api_base_url,
            discord_base_url,
            asset_base_url,
        }
    }
}

fn normalize_base_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branding(instance_url: &str, cdn_url: Option<&str>) -> BrandingConfig {
        BrandingConfig {
            instance_name: "Underground".into(),
            instance_url: instance_url.into(),
            sentry_url: "https://sentry.io".into(),
            status_url: "https://status.discord.com".into(),
            gateway_url: Some("ws://localhost:5001".into()),
            cdn_url: cdn_url.map(str::to_string),
        }
    }

    #[test]
    fn legacy_discord_base_url_matching_backend_is_ignored_for_upstream() {
        let urls = ResolvedUrls::from_config(
            &branding("http://localhost:5002", None),
            Some("http://localhost:5002".into()),
            None,
            None,
        );

        assert_eq!(urls.api_base_url, "http://localhost:5002");
        assert_eq!(urls.discord_base_url, "https://canary.discord.com");
        assert_eq!(urls.asset_base_url, "https://discord.com");
    }

    #[test]
    fn explicit_overrides_win_over_defaults() {
        let urls = ResolvedUrls::from_config(
            &branding("https://api.example.com", Some("https://cdn.example.com/")),
            Some("https://legacy.example.com".into()),
            Some("https://canary.discord.com/".into()),
            Some("https://assets.example.com/".into()),
        );

        assert_eq!(urls.api_base_url, "https://api.example.com");
        assert_eq!(urls.discord_base_url, "https://canary.discord.com");
        assert_eq!(urls.asset_base_url, "https://assets.example.com");
    }

    #[test]
    fn cdn_url_does_not_affect_asset_base_url() {
        // cdn_url is only used for GLOBAL_ENV CDN_HOST injection, not for asset fetching
        let urls = ResolvedUrls::from_config(
            &branding("https://api.example.com", Some("https://cdn.example.com/")),
            None,
            None,
            None,
        );

        assert_eq!(urls.asset_base_url, "https://discord.com");
    }
}
