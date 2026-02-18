use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub async fn serve_static(Path(file): Path<String>) -> Response {
    let base = std::path::Path::new("static");
    let path = base.join(&file);

    // prevent directory traversal
    if file.contains("..") {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let bytes = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let content_type = if file.ends_with(".ttf") || file.ends_with(".TTF") {
        "font/ttf"
    } else if file.ends_with(".woff2") {
        "font/woff2"
    } else if file.ends_with(".woff") {
        "font/woff"
    } else if file.ends_with(".css") {
        "text/css"
    } else if file.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    };

    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            ),
        ],
        bytes,
    )
        .into_response()
}
