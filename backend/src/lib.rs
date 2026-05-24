pub mod pubmed;
pub mod routes;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Build the full Axum app. Used by both `main.rs` and integration tests so
/// tests exercise the exact same wiring.
pub fn app() -> Router {
    let client = pubmed::Client::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/search", get(routes::search))
        .route("/api/article/:pmid", get(routes::article))
        .route("/api/mesh", get(routes::mesh_suggest))
        .route("/api/cite/:pmid", get(routes::cite))
        .with_state(client)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
