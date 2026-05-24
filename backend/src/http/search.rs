//! `GET /api/search` — proxy NCBI esearch + esummary, with timing.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use utoipa::{IntoParams, ToSchema};

use crate::error::{AppError, ErrorResponse};
use crate::infra::ncbi::Summary;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Free-text PubMed query, e.g. `crispr cas9` or `covid 2024[dp]`.
    pub term: String,
    /// 1-based page number.
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Sort order. One of: `relevance`, `pub_date`, `first_author`, `journal`, `title`.
    pub sort: Option<String>,
    /// Comma-separated PubMed filter expressions appended to the term,
    /// e.g. `humans[Filter],english[lang],2020:2025[dp],Review[pt]`.
    pub filters: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub count: u32,
    pub page: u32,
    pub page_size: u32,
    pub query_translation: String,
    /// Server-side wall-clock duration for the upstream NCBI roundtrip,
    /// in milliseconds.
    pub elapsed_ms: u64,
    pub results: Vec<Summary>,
}

#[utoipa::path(
    get,
    path = "/api/search",
    tag = "pubmed",
    params(SearchQuery),
    responses(
        (status = 200, description = "Paginated PubMed search results",
            body = SearchResponse),
        (status = 500, description = "Upstream NCBI error", body = ErrorResponse),
    ),
)]
pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let started = Instant::now();

    let mut term = q.term.trim().to_string();
    if let Some(f) = q.filters.as_ref() {
        for filt in f.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            term = format!("({term}) AND {filt}");
        }
    }
    let retstart = q.page.saturating_sub(1) * q.page_size;

    let es = state
        .ncbi
        .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref())
        .await?;
    let summaries = state.ncbi.esummary("pubmed", &es.ids).await?;

    Ok(Json(SearchResponse {
        count: es.count,
        page: q.page,
        page_size: q.page_size,
        query_translation: es.querytranslation,
        elapsed_ms: started.elapsed().as_millis() as u64,
        results: summaries,
    }))
}
