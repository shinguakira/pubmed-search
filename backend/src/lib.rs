pub mod pubmed;
pub mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "The PubMed Gazette API",
        version = "0.1.0",
        description = "Thin Rust proxy around NCBI E-utilities for the PubMed Gazette frontend.",
    ),
    tags(
        (name = "pubmed", description = "Search, fetch, and cite PubMed articles"),
        (name = "mesh", description = "MeSH term suggestions"),
    ),
)]
pub struct ApiDoc;

/// Returns just the generated OpenAPI document. Used by `bin/gen-openapi.rs`
/// and tests; not strictly needed by `main`.
pub fn openapi() -> utoipa::openapi::OpenApi {
    build_router().1
}

/// Build the full Axum app. Used by both `main.rs` and integration tests so
/// tests exercise the exact same wiring.
pub fn app() -> Router {
    let (router, api) = build_router();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .merge(SwaggerUi::new("/docs").url("/api/openapi.json", api))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// `routes!(name)` here is the static check: a handler without
/// `#[utoipa::path(...)]` simply will not compile.
fn build_router() -> (Router, utoipa::openapi::OpenApi) {
    let client = pubmed::Client::new();
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(routes::search))
        .routes(routes!(routes::article))
        .routes(routes!(routes::mesh_suggest))
        .routes(routes!(routes::cite))
        .with_state(client)
        .split_for_parts()
}
