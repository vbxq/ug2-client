use crate::server::ip;
use crate::server::state::AppState;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use redis::AsyncCommands;
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct RateLimitError {
    status: &'static str,
    message: String,
    retry_after: u32,
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let server_config = &state.config.patch_config.server;

    if !server_config.rate_limit_enabled {
        return next.run(request).await;
    }

    let ip = ip::extract_real_ip(&headers, &connect_info, server_config.trust_proxy_headers);
    let key = format!("rl:{}", ip);
    let window = server_config.rate_limit_window_secs;
    let max_requests = server_config.rate_limit_requests;

    let mut redis = state.redis.clone();

    let count: u32 = match redis.incr::<_, _, u32>(&key, 1u32).await {
        Ok(c) => {
            if c == 1 {
                let _ = redis.expire::<_, ()>(&key, window as i64).await;
            }
            c
        }
        Err(e) => {
            tracing::warn!("Rate limiter Redis error (fail-open): {}", e);
            return next.run(request).await;
        }
    };

    if count > max_requests {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitError {
                status: "error",
                message: "Rate limit exceeded".into(),
                retry_after: window,
            }),
        )
            .into_response();
    }

    next.run(request).await
}
