use crate::asset_downloader::AssetDownloader;
use crate::cache::redis_cache;
use crate::db::models::discord_build;
use crate::discord_scraper::{build_parser, GitHubClient};
use crate::server::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sea_orm::prelude::Expr;
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DownloadRequest {
    pub build_hash: Option<String>,
}

#[derive(Serialize)]
pub struct BuildResponse {
    pub build_hash: String,
    pub channel: String,
    pub is_patched: bool,
    pub is_active: bool,
    pub build_date: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub message: String,
}

struct DownloadInfo {
    build_hash: String,
    channel: String,
    scripts: Vec<String>,
    global_env: serde_json::Value,
    timestamp: i64,
}

// POST /api/builds/download
pub async fn download_build(
    State(state): State<AppState>,
    Json(req): Json<DownloadRequest>,
) -> Response {
    let info = if let Some(hash) = &req.build_hash {
        match discord_build::Entity::find()
            .filter(discord_build::Column::BuildHash.eq(hash))
            .one(&state.db)
            .await
        {
            Ok(Some(build)) => {
                let scripts: Vec<String> =
                    serde_json::from_value(build.scripts).unwrap_or_default();
                tracing::info!("Found build {} in DB with {} scripts", hash, scripts.len());
                DownloadInfo {
                    build_hash: build.build_hash,
                    channel: build.channel,
                    scripts,
                    global_env: build.global_env.unwrap_or(serde_json::json!({})),
                    timestamp: build.build_date.timestamp_millis(),
                }
            }
            _ => {
                let github = GitHubClient::new(&state.config.github_builds_repo);
                match github.fetch_build_by_hash(hash).await {
                    Ok(build_data) => match build_parser::parse_build(&build_data) {
                        Ok(bi) => DownloadInfo {
                            build_hash: bi.build_hash,
                            channel: bi.channel,
                            scripts: bi.scripts,
                            global_env: bi.global_env,
                            timestamp: bi.timestamp,
                        },
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse build: {}", e)),
                    },
                    Err(e) => return error_response(StatusCode::BAD_REQUEST, format!("Build not found: {}", e)),
                }
            }
        }
    } else {
        let github = GitHubClient::new(&state.config.github_builds_repo);
        match github.fetch_latest_build().await {
            Ok((hash, entry)) => match github.fetch_build_at_path(&entry.path, &hash).await {
                Ok(build_data) => match build_parser::parse_build(&build_data) {
                    Ok(bi) => DownloadInfo {
                        build_hash: bi.build_hash,
                        channel: bi.channel,
                        scripts: bi.scripts,
                        global_env: bi.global_env,
                        timestamp: bi.timestamp,
                    },
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse build: {}", e)),
                },
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch build data: {}", e)),
            },
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to find latest build: {}", e)),
        }
    };

    let config = state.config.clone();
    let db = state.db.clone();
    let pipeline = state.pipeline.clone();
    let fs_cache = state.fs_cache.clone();
    let mut redis = state.redis.clone();
    let http_client = state.http_client.clone();
    let build_hash = info.build_hash.clone();
    let resp_hash = build_hash.clone();

    tokio::spawn(async move {
        tracing::info!("Starting download for build {} ({} scripts)", build_hash, info.scripts.len());

        let downloader =
            AssetDownloader::new(http_client, config.cache_path.clone(), &config.discord_base_url);
        match downloader
            .download_build(&build_hash, &info.scripts)
            .await
        {
            Ok(assets) => {
                tracing::info!(
                    "Downloaded {} assets for build {}",
                    assets.len(),
                    build_hash
                );

                let build_dir = fs_cache.build_dir(&build_hash);

                let index_scripts = crate::asset_downloader::detect_entry_scripts(
                    &build_dir,
                    &info.scripts,
                );

                match pipeline.patch_build(&build_dir).await {
                    Ok(count) => {
                        tracing::info!("Patched {} files for build {}", count, build_hash)
                    }
                    Err(e) => tracing::error!("Patching failed for {}: {}", build_hash, e),
                }

                let ts = chrono::DateTime::from_timestamp_millis(info.timestamp)
                    .unwrap_or_default()
                    .fixed_offset();

                let global_env_db = if info.global_env.as_object().map_or(false, |m| m.is_empty()) {
                    None
                } else {
                    Some(info.global_env)
                };

                let active = discord_build::ActiveModel {
                    build_hash: Set(build_hash.clone()),
                    channel: Set(info.channel),
                    build_date: Set(ts),
                    global_env: Set(global_env_db),
                    scripts: Set(serde_json::to_value(&info.scripts).unwrap()),
                    index_scripts: Set(serde_json::to_value(&index_scripts).unwrap()),
                    is_patched: Set(true),
                    is_active: Set(false),
                    ..Default::default()
                };

                if let Err(e) = discord_build::Entity::insert(active)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::column(
                            discord_build::Column::BuildHash,
                        )
                        .update_columns([
                            discord_build::Column::IsPatched,
                            discord_build::Column::IndexScripts,
                            discord_build::Column::GlobalEnv,
                            discord_build::Column::UpdatedAt,
                        ])
                        .to_owned(),
                    )
                    .exec_without_returning(&db)
                    .await
                {
                    tracing::error!("Failed to save build to DB: {}", e);
                }

                let _ = redis_cache::invalidate_build(&mut redis, &build_hash).await;
                let _ = redis_cache::invalidate_builds_cache(&mut redis).await;
                tracing::info!("Build {} ready! Detected {} entry scripts", build_hash, index_scripts.len());
            }
            Err(e) => tracing::error!("Download failed for {}: {}", build_hash, e),
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(StatusResponse {
            status: "accepted".into(),
            message: format!("Download started for build {}", resp_hash),
        }),
    )
        .into_response()
}

// POST /api/builds/fetch-current
pub async fn fetch_current_build(State(state): State<AppState>) -> Response {
    let live = match crate::discord_scraper::fetch_live_build(
        &state.http_client,
        &state.config.discord_base_url,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                format!("Failed to scrape Discord: {}", e),
            )
        }
    };

    let build_hash = live.build_hash.clone();
    let resp_hash = build_hash.clone();

    let index_scripts = live.scripts.clone();

    let config = state.config.clone();
    let db = state.db.clone();
    let pipeline = state.pipeline.clone();
    let fs_cache = state.fs_cache.clone();
    let mut redis = state.redis.clone();
    let http_client = state.http_client.clone();

    tokio::spawn(async move {
        tracing::info!(
            "Fetching current build {} ({} entry scripts)",
            build_hash,
            live.scripts.len()
        );

        let downloader =
            AssetDownloader::new(http_client, config.cache_path.clone(), &config.discord_base_url);
        match downloader
            .download_build(&build_hash, &live.scripts)
            .await
        {
            Ok(assets) => {
                tracing::info!("Downloaded {} assets for build {}", assets.len(), build_hash);

                let build_dir = fs_cache.build_dir(&build_hash);

                match pipeline.patch_build(&build_dir).await {
                    Ok(count) => {
                        tracing::info!("Patched {} files for build {}", count, build_hash)
                    }
                    Err(e) => tracing::error!("Patching failed for {}: {}", build_hash, e),
                }

                let ts = chrono::DateTime::from_timestamp_millis(live.timestamp)
                    .unwrap_or_default()
                    .fixed_offset();

                let active = discord_build::ActiveModel {
                    build_hash: Set(build_hash.clone()),
                    channel: Set(live.channel),
                    build_date: Set(ts),
                    global_env: Set(Some(live.global_env)),
                    scripts: Set(serde_json::to_value(&assets).unwrap()),
                    index_scripts: Set(serde_json::to_value(&index_scripts).unwrap()),
                    is_patched: Set(true),
                    is_active: Set(false),
                    ..Default::default()
                };

                if let Err(e) = discord_build::Entity::insert(active)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::column(
                            discord_build::Column::BuildHash,
                        )
                        .update_columns([
                            discord_build::Column::IsPatched,
                            discord_build::Column::IndexScripts,
                            discord_build::Column::Scripts,
                            discord_build::Column::GlobalEnv,
                            discord_build::Column::UpdatedAt,
                        ])
                        .to_owned(),
                    )
                    .exec_without_returning(&db)
                    .await
                {
                    tracing::error!("Failed to save build to DB: {}", e);
                }

                let _ = redis_cache::invalidate_build(&mut redis, &build_hash).await;
                let _ = redis_cache::invalidate_builds_cache(&mut redis).await;
                tracing::info!("Build {} ready!", build_hash);
            }
            Err(e) => tracing::error!("Download failed for {}: {}", build_hash, e),
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(StatusResponse {
            status: "accepted".into(),
            message: format!(
                "Fetching current build {} from Discord",
                resp_hash
            ),
        }),
    )
        .into_response()
}

fn error_response(status: StatusCode, message: String) -> Response {
    (status, Json(StatusResponse { status: "error".into(), message })).into_response()
}

// GET /api/builds
pub async fn list_builds(State(state): State<AppState>) -> Response {
    let mut redis = state.redis.clone();
    let cache_key = redis_cache::builds_list_key();

    if let Ok(Some(cached)) = redis_cache::get_cached_json(&mut redis, cache_key).await {
        return ([("content-type", "application/json")], cached).into_response();
    }

    match discord_build::Entity::find()
        .order_by_desc(discord_build::Column::BuildDate)
        .all(&state.db)
        .await
    {
        Ok(builds) => {
            let response: Vec<BuildResponse> = builds
                .iter()
                .map(|b| BuildResponse {
                    build_hash: b.build_hash.clone(),
                    channel: b.channel.clone(),
                    is_patched: b.is_patched,
                    is_active: b.is_active,
                    build_date: b.build_date.to_string(),
                })
                .collect();

            if let Ok(json) = serde_json::to_string(&response) {
                let _ = redis_cache::cache_json(&mut redis, cache_key, &json, 30).await;
            }

            Json(response).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)).into_response()
        }
    }
}

// PUT /api/builds/active
#[derive(Deserialize)]
pub struct SetActiveRequest {
    pub build_hash: String,
}

pub async fn set_active_build(
    State(state): State<AppState>,
    Json(req): Json<SetActiveRequest>,
) -> Response {
    let _ = discord_build::Entity::update_many()
        .col_expr(discord_build::Column::IsActive, Expr::value(false))
        .exec(&state.db)
        .await;

    let result = discord_build::Entity::update_many()
        .col_expr(discord_build::Column::IsActive, Expr::value(true))
        .filter(discord_build::Column::BuildHash.eq(&req.build_hash))
        .exec(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected > 0 => {
            *state.active_build.write().await = Some(req.build_hash.clone());

            let mut redis = state.redis.clone();
            let _ =
                crate::cache::redis_cache::invalidate_build(&mut redis, &req.build_hash).await;
            let _ = redis_cache::invalidate_builds_cache(&mut redis).await;

            Json(StatusResponse {
                status: "ok".into(),
                message: format!("Active build set to {}", req.build_hash),
            })
            .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(StatusResponse {
                status: "error".into(),
                message: "Build not found".into(),
            }),
        )
            .into_response(),
    }
}

// PUT /api/builds/{hash}/index-scripts
#[derive(Deserialize)]
pub struct SetIndexScriptsRequest {
    pub index_scripts: Vec<String>,
}

pub async fn set_index_scripts(
    State(state): State<AppState>,
    axum::extract::Path(build_hash): axum::extract::Path<String>,
    Json(req): Json<SetIndexScriptsRequest>,
) -> Response {
    let result = discord_build::Entity::update_many()
        .col_expr(
            discord_build::Column::IndexScripts,
            Expr::value(serde_json::to_value(&req.index_scripts).unwrap()),
        )
        .filter(discord_build::Column::BuildHash.eq(&build_hash))
        .exec(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected > 0 => {
            let mut redis = state.redis.clone();
            let _ = crate::cache::redis_cache::invalidate_build(&mut redis, &build_hash).await;

            Json(StatusResponse {
                status: "ok".into(),
                message: format!(
                    "Set {} index scripts for build {}",
                    req.index_scripts.len(),
                    build_hash
                ),
            })
            .into_response()
        }
        Ok(_) => error_response(StatusCode::NOT_FOUND, "Build not found".into()),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)),
    }
}

// POST /api/builds/{hash}/repatch
pub async fn repatch_build(
    State(state): State<AppState>,
    axum::extract::Path(build_hash): axum::extract::Path<String>,
) -> Response {
    let build_dir = state.fs_cache.build_dir(&build_hash);
    if !build_dir.exists() {
        return error_response(StatusCode::NOT_FOUND, "Build not found in cache".into());
    }

    let pipeline = state.pipeline.clone();
    let db = state.db.clone();
    let mut redis = state.redis.clone();
    let hash = build_hash.clone();

    let scripts: Vec<String> = match discord_build::Entity::find()
        .filter(discord_build::Column::BuildHash.eq(&build_hash))
        .one(&state.db)
        .await
    {
        Ok(Some(build)) => serde_json::from_value(build.scripts).unwrap_or_default(),
        _ => Vec::new(),
    };

    let _ = redis_cache::invalidate_build(&mut redis, &build_hash).await;
    let _ = redis_cache::invalidate_builds_cache(&mut redis).await;

    tokio::spawn(async move {
        match pipeline.patch_build(&build_dir).await {
            Ok(count) => tracing::info!("Repatched {} files for build {}", count, hash),
            Err(e) => tracing::error!("Repatching failed: {}", e),
        }

        if !scripts.is_empty() {
            let index_scripts =
                crate::asset_downloader::detect_entry_scripts(&build_dir, &scripts);

            let _ = discord_build::Entity::update_many()
                .col_expr(
                    discord_build::Column::IndexScripts,
                    Expr::value(serde_json::to_value(&index_scripts).unwrap()),
                )
                .filter(discord_build::Column::BuildHash.eq(&hash))
                .exec(&db)
                .await;
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(StatusResponse {
            status: "accepted".into(),
            message: "Repatch started".into(),
        }),
    )
        .into_response()
}
