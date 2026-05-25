use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::infra::ncbi::{ArticleDetail, Summary};

/// Query parameters for `GET /api/search`.
#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Free-text PubMed query. Field tags (`[ti]`, `[au]`, `[mesh]`,
    /// `[dp]`, …) and boolean operators (`AND`, `OR`, `NOT`) work as on
    /// PubMed itself. Example: `crispr cas9` or `covid 2024[dp]`.
    pub term: String,

    /// 1-based page number. NCBI internally uses 0-based offsets; the
    /// handler converts.
    #[serde(default = "default_page")]
    pub page: u32,

    /// Number of results per page. NCBI's hard ceiling is 10000 but
    /// practical UIs stay ≤ 100.
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// Sort order. One of: `relevance`, `pub_date`, `first_author`,
    /// `journal`, `title`. Omitted = NCBI default (relevance).
    pub sort: Option<String>,

    /// Comma-separated raw PubMed filter expressions appended to `term`
    /// as `(term) AND filter1 AND filter2 …`. Each fragment uses the
    /// same `[tag]` syntax PubMed expects, e.g.
    /// `humans[Filter],english[lang],2020:2025[dp],Review[pt]`.
    pub filters: Option<String>,

    /// When `true`, the handler hits NCBI via `esearch(usehistory=y)` +
    /// `efetch_bulk` instead of `esearch` + `esummary`, and populates
    /// the `details` field of the response with full
    /// `ArticleDetail` records. Clients can prewarm a per-PMID cache
    /// off this so subsequent article-detail clicks are instant.
    ///
    /// Trade-off: the initial response is slightly slower and heavier
    /// (XML parse vs JSON), but the cumulative time across a "search →
    /// open several articles" flow is dramatically lower.
    #[serde(default)]
    pub bulk: bool,

    /// App-level **post-filter** keyword. Applied by *this* backend
    /// after NCBI has already returned the page. Independent from the
    /// PubMed query language — pure case-insensitive substring against
    /// `title + abstract + authors + journal`. Empty / absent = no
    /// filter.
    #[serde(default)]
    pub app_filter: Option<String>,

    /// How to interpret matches against `app_filter`:
    /// * `"include"` — keep only rows that match.
    /// * `"exclude"` — drop rows that match (default).
    #[serde(default)]
    pub app_filter_mode: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    100
}

/// Response body for `GET /api/search`.
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    /// Total matching records in PubMed for this query (the whole corpus,
    /// not just the current page).
    pub count: u32,

    /// Echo of the requested page number (1-based).
    pub page: u32,

    /// Echo of the requested page size.
    pub page_size: u32,

    /// The query as NCBI rewrote it after MeSH expansion and synonym
    /// substitution.
    pub query_translation: String,

    /// Server-side wall-clock duration for the upstream NCBI roundtrip(s),
    /// in milliseconds.
    pub elapsed_ms: u64,

    /// Page slice of citation summaries — always the rendering shape.
    /// Built either from `esummary` (default) or from `efetch_bulk`
    /// output mapped down to Summary fields (bulk mode).
    pub results: Vec<Summary>,

    /// Full article records for the same page slice. Populated only
    /// when the request asked for `bulk=true`. Same length and PMID
    /// order as `results`. Use to prewarm a client-side article cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<ArticleDetail>>,

    /// Size of the page slice **before** the app-filter was applied.
    /// Only populated when `app_filter` was a non-empty term — clients
    /// use it to render a "N / M shown after app filter" badge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unfiltered_count: Option<u32>,
}
