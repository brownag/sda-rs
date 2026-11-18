pub mod config;
pub mod executor;
pub mod request;
pub mod result;
pub mod routes;
pub mod serializer;
pub mod transpiler;

use axum::{routing::post, Router};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub database_path: String,
}

/// Build the HTTP router. Database and spatial initialization are request-layer concerns.
pub fn app(config: config::Config) -> Router {
    let state = AppState {
        database_path: config.backend_url,
    };
    Router::new()
        .route("/Tabular/post.rest", post(routes::post))
        .route("/Tab/post.rest", post(routes::post))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
