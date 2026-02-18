pub mod handlers;
pub mod ip;
pub mod rate_limit;
pub mod routes;
pub mod state;

use crate::cache::FsCache;
use crate::config::AppConfig;
use crate::db::models::discord_build;
use crate::patcher::PatchPipeline;
use anyhow::Result;
use redis::aio::ConnectionManager;
use sea_orm::*;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn run(
    config: AppConfig,
    db: DatabaseConnection,
    redis: ConnectionManager,
) -> Result<()> {
    crate::db::run_migrations(&db).await?;

    let fs_cache = Arc::new(FsCache::new(config.cache_path.clone()));
    let pipeline = Arc::new(PatchPipeline::new(&config.patch_config));

    let active_build = discord_build::Entity::find()
        .filter(discord_build::Column::IsActive.eq(true))
        .one(&db)
        .await?
        .map(|b| b.build_hash);

    if let Some(ref hash) = active_build {
        tracing::info!("Active build: {}", hash);
    } else {
        tracing::warn!("No active build set. Use PUT /api/builds/active to set one.");
    }

    let state = AppState {
        config: config.clone(),
        db,
        redis,
        fs_cache,
        pipeline,
        active_build: Arc::new(RwLock::new(active_build)),
        http_client: reqwest::Client::new(),
        proxy_semaphore: Arc::new(tokio::sync::Semaphore::new(50)),
    };

    let app = routes::build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("Server listening on {}", config.bind_addr);

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutdown signal received, finishing active requests...");
}
