use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::infra::ncbi::Summary;

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
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
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
    /// substitution. Useful for showing the user "we actually searched
    /// for X" beneath the search box.
    pub query_translation: String,

    /// Server-side wall-clock duration for the upstream NCBI roundtrip
    /// (esearch + esummary), in milliseconds. Network latency from the
    /// browser to this server is *not* included.
    pub elapsed_ms: u64,

    /// Page slice of citation summaries (always ≤ `page_size`).
    pub results: Vec<Summary>,
}
