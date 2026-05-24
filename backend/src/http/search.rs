//! `GET /api/search` — two paths:
//! * **default**: esearch + esummary (fast, lightweight Summary).
//! * **bulk=true**: esearch(usehistory=y) + efetch_bulk (heavier, but
//!   returns full ArticleDetail per record so clients can prewarm an
//!   article cache and make detail clicks instant).

use axum::extract::{Query, State};
use axum::Json;
use std::time::Instant;

use crate::error::AppError;
use crate::http::dto::error::ErrorResponse;
use crate::http::dto::search::{SearchQuery, SearchResponse};
use crate::infra::ncbi::{ArticleDetail, Summary};
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

    let (results, details, count, query_translation) = if q.bulk {
        // Bulk path: esearch with History → efetch_bulk for the page slice.
        let es = state
            .ncbi
            .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref(), true)
            .await?;
        let details = if es.ids.is_empty() {
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
                .efetch_bulk(&web_env, query_key, 0, q.page_size)
                .await?
        };
        // Populate the process-local article cache so a subsequent
        // /api/article/{pmid} call for any PMID on this page is served
        // from memory in microseconds. *This* is where the bulk speedup
        // lands for the user — frontend stays dumb.
        state.articles.put_many(details.iter().cloned());
        // Map ArticleDetail → Summary for the list view.
        let summaries: Vec<Summary> = details.iter().map(summary_from_detail).collect();
        (summaries, Some(details), es.count, es.querytranslation)
    } else {
        // Default path: esearch + esummary (cheap).
        let es = state
            .ncbi
            .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref(), false)
            .await?;
        let summaries = state.ncbi.esummary("pubmed", &es.ids).await?;
        (summaries, None, es.count, es.querytranslation)
    };

    Ok(Json(SearchResponse {
        count,
        page: q.page,
        page_size: q.page_size,
        query_translation,
        elapsed_ms: started.elapsed().as_millis() as u64,
        results,
        details,
    }))
}

/// Build the list-row `Summary` from a `efetch` `ArticleDetail`.
/// Fields not carried in efetch XML (epubdate, volume, issue, pages,
/// lang, short author form, journal abbreviation) are left empty or
/// derived as best we can — esummary is still the better source for
/// those, which is why default search uses esummary.
fn summary_from_detail(d: &ArticleDetail) -> Summary {
    let authors_short: Vec<String> = d
        .authors
        .iter()
        .map(|a| {
            let initials: String = a
                .fore_name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect();
            format!("{} {}", a.last_name, initials).trim().to_string()
        })
        .collect();
    Summary {
        pmid: d.pmid.clone(),
        title: d.title.clone(),
        authors: authors_short,
        source: d.journal.clone(),
        pubdate: d.pubdate.clone(),
        epubdate: String::new(),
        volume: String::new(),
        issue: String::new(),
        pages: String::new(),
        doi: d.doi.clone(),
        pubtypes: d.pubtypes.clone(),
        lang: String::new(),
    }
}
