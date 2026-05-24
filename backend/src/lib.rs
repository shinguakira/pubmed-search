//! Library crate. Holds the router builder, the OpenAPI document, and the
//! NCBI client. Lives in `lib.rs` (not `main.rs`) so that:
//!
//! * `bin/gen-openapi.rs` can call `openapi()` to emit `docs/openapi.json`,
//! * `tests/api.rs` can call `app()` to spawn the real server on an
//!   ephemeral port and hit live NCBI without mocks,
//! * the binary in `main.rs` simply re-uses `app()` too.
//!
//! All HTTP wiring (routes, CORS, tracing, Swagger UI) is centralized here.

pub mod pubmed;
pub mod routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

/// `#[derive(OpenApi)]` makes utoipa generate an OpenAPI 3.1 document from
/// the `#[utoipa::path]` annotations on the handlers below. The struct
/// itself is empty — it only exists so the macro has a type to attach to.
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

/// Returns just the generated OpenAPI document, no HTTP server.
/// Used by `bin/gen-openapi.rs` and `tests/openapi.rs`.
pub fn openapi() -> utoipa::openapi::OpenApi {
    build_router().1
}

/// Build the full Axum app: handlers + Swagger UI + CORS + tracing.
///
/// Both `main.rs` and the integration tests call this so they exercise the
/// exact same wiring — anything that compiles + works in tests is what
/// production runs.
pub fn app() -> Router {
    let (router, api) = build_router();

    // We allow any origin / method / header. This is a localhost-only PoC;
    // tighten this if it ever ships somewhere public.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router
        // Plain health probe (used by Playwright's webServer readiness check).
        .route("/api/health", axum::routing::get(|| async { "ok" }))
        // Mount Swagger UI at /docs, serving the live spec at /api/openapi.json.
        // No build step or static files needed — the JSON is generated at
        // process start from the same `routes!()` calls below.
        .merge(SwaggerUi::new("/docs").url("/api/openapi.json", api))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// Where the static "every handler must be documented" check happens.
///
/// `routes!(routes::search)` is a macro from `utoipa_axum`. It reads the
/// HTTP method and URL path out of the handler's `#[utoipa::path(...)]`
/// attribute and registers it on both the Axum router AND the OpenAPI
/// document at the same time. **A handler without `#[utoipa::path]`
/// simply will not compile when added here.** That is the compile-time
/// enforcement utoipa provides for free.
fn build_router() -> (Router, utoipa::openapi::OpenApi) {
    // `pubmed::Client` is the NCBI HTTP client; it is shared by all handlers
    // via Axum's `State` extractor (see `routes.rs`).
    let client = pubmed::Client::new();

    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(routes::search))
        .routes(routes!(routes::article))
        .routes(routes!(routes::mesh_suggest))
        .routes(routes!(routes::cite))
        // `with_state` injects the shared `Client` into every handler.
        .with_state(client)
        // Returns `(Router, OpenApi)` — the router goes to Axum, the spec
        // to Swagger UI / the JSON endpoint.
        .split_for_parts()
}
