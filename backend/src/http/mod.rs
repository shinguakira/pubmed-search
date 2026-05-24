//! HTTP boundary layer.
//!
//! One module per resource. Each module owns:
//! * its DTOs (request `*Query` / response `*Response`),
//! * the handler `pub async fn`,
//! * the `#[utoipa::path(...)]` annotation.
//!
//! `build` assembles them into a single `(Router, OpenApi)` pair. The
//! `routes!(handler)` macro from `utoipa_axum` is the compile-time check:
//! a handler without `#[utoipa::path]` will not compile when referenced.

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::state::AppState;

pub mod article;
pub mod cite;
pub mod mesh;
pub mod search;

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

pub fn build(state: AppState) -> (Router, utoipa::openapi::OpenApi) {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(search::search))
        .routes(routes!(article::article))
        .routes(routes!(mesh::mesh_suggest))
        .routes(routes!(cite::cite))
        .with_state(state)
        .split_for_parts()
}
