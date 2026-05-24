//! Library crate. Top-level wiring lives here; everything else is in
//! sibling modules.
//!
//! Layer map:
//! * `http`   — HTTP boundary (handlers, DTOs, `#[utoipa::path]`).
//! * `domain` — pure logic (currently empty; reserved for citation,
//!   query-building, etc. that should be unit-testable without IO).
//! * `infra`  — external IO (NCBI client today; DB / cache later).
//! * `state`  — `AppState` (shared dependencies injected into handlers).
//! * `error`  — `AppError` + `IntoResponse` (single mapping to HTTP).
//!
//! Three callers re-use the wiring built here:
//! * `main.rs`              — production binary.
//! * `bin/gen-openapi.rs`   — dumps `docs/openapi.json`.
//! * `tests/api.rs`         — spawns the real server on an ephemeral port
//!   and hits live NCBI without mocks.

pub mod error;
pub mod http;
pub mod infra;
pub mod state;

pub use http::ApiDoc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

/// Returns just the generated OpenAPI document, no HTTP server.
/// Used by `bin/gen-openapi.rs` and `tests/openapi.rs`.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, api) = http::build(AppState::new());
    api
}

/// Build the full Axum app: handlers + Swagger UI + CORS + tracing.
///
/// Both `main.rs` and the integration tests call this so they exercise
/// the exact same wiring.
pub fn app() -> Router {
    let state = AppState::new();
    let (router, api) = http::build(state);

    // Wide-open CORS — localhost-only PoC. Tighten this before shipping.
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
