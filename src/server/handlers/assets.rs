use crate::cache::redis_cache;
use crate::server::state::AppState;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

pub async fn serve_asset(
    State(state): State<AppState>,
    Path(asset_name): Path<String>,
) -> Response {
    let active_build = state.active_build.read().await;
    let build_hash = match active_build.as_ref() {
        Some(h) => h.clone(),
        None => return (StatusCode::SERVICE_UNAVAILABLE, "No active build").into_response(),
    };
    drop(active_build);

    let cache_headers = asset_cache_headers(&asset_name);

    let cache_key = redis_cache::asset_key(&build_hash, &asset_name);
    let mut redis = state.redis.clone();
    if let Ok(Some(data)) = redis_cache::get_cached_asset(&mut redis, &cache_key).await {
        return (cache_headers, data).into_response();
    }

    if let Ok(Some(data)) = state.fs_cache.get_asset(&build_hash, &asset_name).await {
        let _ = redis_cache::cache_asset(&mut redis, &cache_key, &data).await;
        return (cache_headers, data).into_response();
    }

    let url = format!("{}/assets/{}", state.config.discord_base_url, asset_name);
    match state.http_client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(bytes) = resp.bytes().await {
                let is_patchable = asset_name.ends_with(".js") || asset_name.ends_with(".css");
                let data = if is_patchable {
                    let content = String::from_utf8_lossy(&bytes);
                    let patched = state.pipeline.patch_content(&content);
                    patched.into_bytes()
                } else {
                    bytes.to_vec()
                };
                let _ = redis_cache::cache_asset(&mut redis, &cache_key, &data).await;
                let _ = state.fs_cache.put_asset(&build_hash, &asset_name, &data).await;
                return (cache_headers, data).into_response();
            }
        }
        _ => {}
    }

    (StatusCode::NOT_FOUND, "Asset not found").into_response()
}

fn asset_cache_headers(name: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, guess_content_type(name).parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "public, max-age=31536000, immutable".parse().unwrap());
    headers
}

fn guess_content_type(name: &str) -> &'static str {
    if name.ends_with(".js") {
        "application/javascript"
    } else if name.ends_with(".css") {
        "text/css"
    } else if name.ends_with(".svg") {
        "image/svg+xml"
    } else if name.ends_with(".png") {
        "image/png"
    } else if name.ends_with(".woff2") {
        "font/woff2"
    } else if name.ends_with(".woff") {
        "font/woff"
    } else if name.ends_with(".wasm") {
        "application/wasm"
    } else if name.ends_with(".map") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}
