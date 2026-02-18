use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::net::SocketAddr;

pub fn extract_real_ip(
    headers: &HeaderMap,
    connect_info: &ConnectInfo<SocketAddr>,
    trust_proxy_headers: bool,
) -> String {
    if trust_proxy_headers {
        // get the real visitor IP if we put this under cloudflare, but i should probably talk about that in the README
        if let Some(cf_ip) = headers
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok())
        {
            return cf_ip.trim().to_string();
        }

        if let Some(xff) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(first) = xff.split(',').next() {
                let ip = first.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }

        if let Some(real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            return real_ip.trim().to_string();
        }
    }

    connect_info.0.ip().to_string()
}
