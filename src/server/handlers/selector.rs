use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub async fn serve_selector() -> Response {
    match tokio::fs::read_to_string("static/selector.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "selector.html not found").into_response(),
    }
}
