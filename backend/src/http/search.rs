//! `GET /api/search` — proxy NCBI esearch + esummary, with timing.

use axum::extract::{Query, State};
use axum::Json;
use std::time::Instant;

use crate::error::AppError;
use crate::http::dto::error::ErrorResponse;
use crate::http::dto::search::{SearchQuery, SearchResponse};
use crate::state::AppState;

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
        .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref(), false)
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
