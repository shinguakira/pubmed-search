//! `GET /api/search` — two paths, **same returned data**:
//! * **default**: esearch + `efetch` with the PMIDs packed into the
//!   URL (`?id=p1,p2,…`). One round-trip for the whole page.
//! * **bulk=true**: esearch(usehistory=y) + `efetch_bulk` against the
//!   History server (`WebEnv` + `QueryKey`). Also one round-trip for
//!   the whole page.
//!
//! Both paths populate `Summary.abstract_text` and warm
//! `state.articles`. The toggle compares the two NCBI access methods:
//! id-list-in-URL vs WebEnv cursor.

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
        state.articles.put_many(details.iter().cloned());
        let summaries: Vec<Summary> = details.iter().map(summary_from_detail).collect();
        (summaries, Some(details), es.count, es.querytranslation)
    } else {
        // Default path: esearch + a single efetch with the PMIDs packed
        // into the URL. Same number of NCBI hops as bulk; only the way
        // the ID set is conveyed differs.
        let es = state
            .ncbi
            .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref(), false)
            .await?;
        let details = state.ncbi.efetch_by_ids(&es.ids).await?;
        state.articles.put_many(details.iter().cloned());
        let summaries: Vec<Summary> = details.iter().map(summary_from_detail).collect();
        (summaries, Some(details), es.count, es.querytranslation)
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
        abstract_text: Some(d.abstract_text.clone()),
    }
}
