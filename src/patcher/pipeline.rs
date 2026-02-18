use crate::config::PatchConfig;
use anyhow::Result;
use std::path::Path;

pub trait Patch: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, content: &str) -> String;
}

pub struct PatchPipeline {
    patches: Vec<Box<dyn Patch>>,
}

impl PatchPipeline {
    pub fn new(config: &PatchConfig) -> Self {
        use super::patches;
        let mut pipeline = Self { patches: Vec::new() };
        let name = &config.branding.instance_name;

        if config.patches.nitro_rebranding {
            pipeline.patches.push(Box::new(patches::branding::NitroRebranding::new(name)));
        }
        if config.patches.discord_rebranding {
            pipeline.patches.push(Box::new(patches::branding::DiscordRebranding::new(name)));
        }
        if config.patches.title_rebranding {
            pipeline.patches.push(Box::new(patches::branding::TitleRebranding::new(name)));
        }
        if config.patches.server_to_guild {
            pipeline.patches.push(Box::new(patches::branding::ServerToGuild));
        }
        if config.patches.sentry_redirect {
            pipeline.patches.push(Box::new(patches::infrastructure::SentryRedirect::new(&config.branding.sentry_url)));
        }
        if config.patches.status_page_redirect {
            pipeline.patches.push(Box::new(patches::infrastructure::StatusPageRedirect::new(&config.branding.status_url)));
        }
        if config.patches.prevent_localstorage_deletion {
            pipeline.patches.push(Box::new(patches::features::PreventLocalStorageDeletion));
        }
        if config.patches.fast_identify {
            pipeline.patches.push(Box::new(patches::features::FastIdentifyFix));
        }
        if config.patches.gateway_reconnect {
            pipeline.patches.push(Box::new(patches::features::GatewayReconnectPatch));
        }
        if config.patches.remove_qr_login {
            pipeline.patches.push(Box::new(patches::features::RemoveQrCodeLogin));
        }
        if config.patches.no_xss_warning {
            pipeline.patches.push(Box::new(patches::features::NoXssWarning));
        }
        if config.patches.enable_dev_experiments {
            pipeline.patches.push(Box::new(patches::experiments::EnableDevExperiments));
        }

        tracing::info!("Patch pipeline initialized with {} patches", pipeline.patches.len());
        pipeline
    }

    pub fn patch_content(&self, content: &str) -> String {
        let mut result = content.to_string();
        for patch in &self.patches {
            result = patch.apply(&result);
        }
        result
    }

    pub async fn patch_build(&self, build_dir: &Path) -> Result<u32> {
        let mut count = 0u32;
        let mut entries = tokio::fs::read_dir(build_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if name.ends_with(".js") || name.ends_with(".css") {
                let content = tokio::fs::read_to_string(&path).await?;
                let patched = self.patch_content(&content);
                if patched != content {
                    tokio::fs::write(&path, patched).await?;
                    count += 1;
                    tracing::debug!("Patched: {}", name);
                }
            }
        }

        tracing::info!("Patched {} files in {:?}", count, build_dir);
        Ok(count)
    }
}
