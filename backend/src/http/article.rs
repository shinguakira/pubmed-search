//! `GET /api/article/{pmid}` — proxy NCBI efetch (XML → JSON).

use axum::extract::{Path, State};
use axum::Json;

use crate::error::AppError;
use crate::http::dto::error::ErrorResponse;
use crate::infra::ncbi::ArticleDetail;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/article/{pmid}",
    tag = "pubmed",
    params(("pmid" = String, Path, description = "PubMed ID")),
    responses(
        (status = 200, description = "Structured article detail (abstract, authors, MeSH, …)",
            body = ArticleDetail),
        (status = 500, description = "Upstream NCBI error", body = ErrorResponse),
    ),
)]
pub async fn article(
    State(state): State<AppState>,
    Path(pmid): Path<String>,
) -> Result<Json<ArticleDetail>, AppError> {
    // Fast path: served from the in-memory cache populated by a prior
    // bulk search. No NCBI round-trip.
    if let Some(cached) = state.articles.get(&pmid) {
        return Ok(Json(cached));
    }
    // Miss: hit NCBI and store the result so the next caller for the
    // same PMID is fast.
    let detail = state.ncbi.efetch_abstract(&pmid).await?;
    state.articles.put(detail.clone());
    Ok(Json(detail))
}
