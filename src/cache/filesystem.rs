use anyhow::Result;
use std::path::PathBuf;

pub struct FsCache {
    base_path: PathBuf,
}

impl FsCache {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn build_dir(&self, build_hash: &str) -> PathBuf {
        self.base_path.join(build_hash)
    }

    pub async fn get_asset(&self, build_hash: &str, asset_name: &str) -> Result<Option<Vec<u8>>> {
        let path = self.base_path.join(build_hash).join(asset_name);
        if path.exists() {
            let data = tokio::fs::read(&path).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub async fn put_asset(&self, build_hash: &str, asset_name: &str, data: &[u8]) -> Result<()> {
        let dir = self.base_path.join(build_hash);
        tokio::fs::create_dir_all(&dir).await?;
        let path = dir.join(asset_name);
        tokio::fs::write(&path, data).await?;
        Ok(())
    }

    pub fn build_exists(&self, build_hash: &str) -> bool {
        self.base_path.join(build_hash).exists()
    }

    pub async fn list_builds(&self) -> Result<Vec<String>> {
        let mut builds = Vec::new();
        if self.base_path.exists() {
            let mut entries = tokio::fs::read_dir(&self.base_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        builds.push(name.to_string());
                    }
                }
            }
        }
        Ok(builds)
    }
}
