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
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
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
///
/// In production (`STATIC_DIR` env var pointing at the built frontend),
/// non-API requests fall through to a `ServeDir` that serves the SPA
/// bundle, with `index.html` as the fallback for client-side routes.
pub fn app() -> Router {
    let state = AppState::new();
    let (router, api) = http::build(state);

    // Wide-open CORS — same-origin in production, dev frontend in
    // development. Cheap to keep open since this PoC has no auth.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = router
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        .merge(SwaggerUi::new("/docs").url("/api/openapi.json", api));

    // Serve the built frontend bundle if `STATIC_DIR` is set (production
    // container). Local `cargo run` skips this, so the Vite dev server
    // keeps owning the frontend during development.
    if let Ok(dir) = std::env::var("STATIC_DIR") {
        let dir = PathBuf::from(dir);
        let index = dir.join("index.html");
        // `ServeDir::not_found_service(ServeFile::new(index))` is the SPA
        // fallback: any path the dir doesn't know about gets index.html so
        // React Router can resolve it client-side.
        let static_service = ServeDir::new(&dir).not_found_service(ServeFile::new(index));
        router = router.fallback_service(static_service);
    }

    router.layer(cors).layer(TraceLayer::new_for_http())
}
