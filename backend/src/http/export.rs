//! `GET /api/search/export` — bulk-fetch search results in one round-
//! trip via NCBI's History server (`usehistory=y` + `efetch` with
//! `WebEnv`/`QueryKey`) and render them as BibTeX / CSV / JSON.

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::domain::citation;
use crate::error::AppError;
use crate::http::dto::error::ErrorResponse;
use crate::http::dto::export::{ExportFormat, ExportQuery};
use crate::state::AppState;

/// NCBI's hard ceiling per `efetch` call via the History server.
const MAX_RECORDS_PER_CALL: u32 = 10_000;

#[utoipa::path(
    get,
    path = "/api/search/export",
    tag = "pubmed",
    params(ExportQuery),
    responses(
        (status = 200, description =
            "Bulk export of the search result set. Response body type \
             depends on the requested `format`: BibTeX text, CSV text, \
             or a JSON array of ArticleDetail. The Content-Type header \
             reflects this."),
        (status = 500, description = "Upstream NCBI error", body = ErrorResponse),
    ),
)]
pub async fn export(
    State(state): State<AppState>,
    Query(q): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let max = q.max.min(MAX_RECORDS_PER_CALL);

    let mut term = q.term.trim().to_string();
    if let Some(f) = q.filters.as_ref() {
        for filt in f.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            term = format!("({term}) AND {filt}");
        }
    }

    // 1. esearch with usehistory=y → WebEnv + QueryKey (and the count).
    let es = state
        .ncbi
        .esearch("pubmed", &term, 0, max, q.sort.as_deref(), true)
        .await?;

    // 2. efetch_bulk for `max` records in one HTTP call (or skip if empty).
    let articles = if es.ids.is_empty() {
        Vec::new()
    } else {
        let web_env = es
            .web_env
            .ok_or_else(|| anyhow::anyhow!("NCBI did not return WebEnv"))?;
        let query_key = es
            .query_key
            .ok_or_else(|| anyhow::anyhow!("NCBI did not return QueryKey"))?;
        state
            .ncbi
            .efetch_bulk(&web_env, query_key, 0, max)
            .await?
    };

    // 3. Render into the requested format.
    let (body, content_type, filename) = match q.format {
        ExportFormat::Bibtex => (
            citation::to_bibtex(&articles),
            "application/x-bibtex; charset=utf-8",
            "pubmed.bib",
        ),
        ExportFormat::Csv => (
            citation::to_csv(&articles),
            "text/csv; charset=utf-8",
            "pubmed.csv",
        ),
        ExportFormat::Json => (
            serde_json::to_string_pretty(&articles).map_err(anyhow::Error::from)?,
            "application/json; charset=utf-8",
            "pubmed.json",
        ),
    };

    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];
    Ok((headers, body).into_response())
}
