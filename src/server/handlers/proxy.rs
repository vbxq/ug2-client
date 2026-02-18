use crate::server::state::AppState;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use reqwest::Url;

pub async fn discord_api_proxy(State(state): State<AppState>, request: Request) -> Response {
    let _permit = match state.proxy_semaphore.acquire().await {
        Ok(p) => p,
        Err(_) => {
            return rate_limit_json(5.0).into_response();
        }
    };

    let path = request.uri().path();
    let query = request
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    let url = format!("{}/api{}{}", state.config.discord_base_url, path, query);

    let method = request.method().clone();
    let req_headers = request.headers().clone();

    let body_bytes = match axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Request body too large").into_response(),
    };

    let mut builder = state.http_client.request(method, &url);

    let discord_base = &state.config.discord_base_url;

    for (name, value) in &req_headers {
        match name {
            &header::HOST | &header::CONNECTION | &header::TRANSFER_ENCODING => continue,
            &header::ORIGIN => {
                builder = builder.header("origin", discord_base.as_str());
            }
            &header::REFERER => {
                if let Ok(v) = value.to_str() {
                    let rewritten = rewrite_origin(v, discord_base);
                    builder = builder.header("referer", rewritten);
                }
            }
            _ => {
                if let Ok(v) = value.to_str() {
                    builder = builder.header(name.as_str(), v);
                }
            }
        }
    }

    if !body_bytes.is_empty() {
        builder = builder.body(body_bytes.to_vec());
    }

    if let Ok(parsed) = Url::parse(&url) {
        if let Some(host) = parsed.host_str() {
            let host_val = if let Some(port) = parsed.port() {
                format!("{}:{}", host, port)
            } else {
                host.to_string()
            };
            builder = builder.header("host", host_val);
        }
    }

    match builder.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);

            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(5.0);

                return rate_limit_json(retry_after).into_response();
            }

            let mut headers = axum::http::HeaderMap::new();
            for (name, value) in resp.headers() {
                match name.as_str() {
                    "transfer-encoding" | "connection" | "keep-alive" => continue,
                    "access-control-allow-origin"
                    | "access-control-allow-methods"
                    | "access-control-allow-headers"
                    | "access-control-allow-credentials" => continue,
                    "set-cookie" => {
                        if let Ok(v) = value.to_str() {
                            let rewritten = rewrite_set_cookie(v);
                            if let Ok(hv) =
                                axum::http::header::HeaderValue::from_str(&rewritten)
                            {
                                headers.append(header::SET_COOKIE, hv);
                            }
                        }
                    }
                    _ => {
                        if let (Ok(n), Ok(v)) = (
                            axum::http::header::HeaderName::from_bytes(name.as_str().as_bytes()),
                            axum::http::header::HeaderValue::from_bytes(value.as_bytes()),
                        ) {
                            headers.insert(n, v);
                        }
                    }
                }
            }

            let body = resp.bytes().await.unwrap_or_default();
            (status, headers, body.to_vec()).into_response()
        }
        Err(e) => {
            tracing::error!("Discord API proxy error: {}", e);
            (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
        }
    }
}

fn rate_limit_json(retry_after: f64) -> Response {
    let body = format!(
        r#"{{"message":"You are being rate limited.","retry_after":{},"global":false}}"#,
        retry_after
    );
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(
        header::HeaderName::from_static("retry-after"),
        format!("{}", retry_after.ceil() as u64).parse().unwrap(),
    );
    (StatusCode::TOO_MANY_REQUESTS, headers, body).into_response()
}

fn rewrite_origin(original: &str, discord_base: &str) -> String {
    match Url::parse(original) {
        Ok(parsed) => {
            let path_and_rest = &original[parsed.origin().ascii_serialization().len()..];
            format!("{}{}", discord_base.trim_end_matches('/'), path_and_rest)
        }
        Err(_) => discord_base.to_string(),
    }
}

fn rewrite_set_cookie(cookie: &str) -> String {
    cookie
        .split(';')
        .map(|part| part.trim())
        .filter(|part| {
            let lower = part.to_lowercase();
            !lower.starts_with("domain=") && lower != "secure"
        })
        .map(|part| {
            if part.to_lowercase().starts_with("samesite=none") {
                "SameSite=Lax"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}
