use crate::cache::FsCache;
use crate::config::AppConfig;
use crate::patcher::PatchPipeline;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
    pub fs_cache: Arc<FsCache>,
    pub pipeline: Arc<PatchPipeline>,
    pub active_build: Arc<RwLock<Option<String>>>,
    pub http_client: reqwest::Client,
    pub proxy_semaphore: Arc<Semaphore>,
}
