pub mod config;
pub mod executor;
pub mod result;
pub mod transpiler;

use axum::{http::StatusCode, routing::post, Router};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};

async fn not_ready() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_IMPLEMENTED,
        "SDA query execution is not configured",
    )
}

/// Build the HTTP router. Database and spatial initialization are request-layer concerns.
pub fn app() -> Router {
    Router::new()
        .route("/Tabular/post.rest", post(not_ready))
        .route("/Tab/post.rest", post(not_ready))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
