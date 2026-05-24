//! `GET /api/article/{pmid}` — proxy NCBI efetch (XML → JSON).

use axum::extract::{Path, State};
use axum::Json;

use crate::error::{AppError, ErrorResponse};
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
    let detail = state.ncbi.efetch_abstract(&pmid).await?;
    Ok(Json(detail))
}
