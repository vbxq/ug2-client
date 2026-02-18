use super::handlers;
use super::rate_limit::rate_limit_middleware;
use super::state::AppState;
use axum::middleware;
use axum::routing::{get, post, put};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

pub fn build_router(state: AppState) -> Router {
    let api_proxy = state.config.patch_config.patches.api_proxy;

    let mut api_router = Router::new()
        .route("/builds", get(handlers::api::list_builds))
        .route("/builds/download", post(handlers::api::download_build))
        .route(
            "/builds/fetch-current",
            post(handlers::api::fetch_current_build),
        )
        .route("/builds/active", put(handlers::api::set_active_build))
        .route(
            "/builds/{hash}/index-scripts",
            put(handlers::api::set_index_scripts),
        )
        .route(
            "/builds/{hash}/repatch",
            post(handlers::api::repatch_build),
        );

    if api_proxy {
        tracing::info!("API proxy enabled — /api/* will be forwarded to {}", state.config.discord_base_url);
        api_router = api_router.fallback(handlers::proxy::discord_api_proxy);
    }

    api_router = api_router
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware));

    Router::new()
        .route("/", get(handlers::index::serve_index))
        .route("/app", get(handlers::index::serve_index))
        .route("/selector", get(handlers::selector::serve_selector))
        .route("/channels/{*tail}", get(handlers::index::serve_index))
        .route("/assets/{asset}", get(handlers::assets::serve_asset))
        .route("/static/{file}", get(handlers::static_files::serve_static))
        .nest("/api", api_router)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}
