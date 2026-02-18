use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub bind_addr: String,
    pub discord_base_url: String,
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

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            discord_base_url: std::env::var("DISCORD_BASE_URL").unwrap_or_else(|_| "https://discord.com".into()),
            github_builds_repo: std::env::var("GITHUB_BUILDS_REPO").unwrap_or_else(|_| "Discord-Build-Logger/Builds".into()),
            cache_path: PathBuf::from(std::env::var("CACHE_PATH").unwrap_or_else(|_| "./assets/cache".into())),
            patch_config,
        })
    }
}
