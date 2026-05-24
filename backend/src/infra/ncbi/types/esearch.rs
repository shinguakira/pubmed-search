use serde::Serialize;

/// Result of an `esearch.fcgi` call.
///
/// Not (currently) exposed in the public HTTP API — `http::dto::search`
/// reshapes this together with `esummary` output into `SearchResponse`.
/// Kept `pub` so other infra modules or future internal endpoints can
/// reuse it.
#[derive(Debug, Serialize)]
pub struct EsearchResult {
    /// Total matching records in NCBI for the query (across all pages).
    pub count: u32,

    /// PMIDs for the current page slice (length ≤ requested `retmax`).
    pub ids: Vec<String>,

    /// NCBI's expansion of the query (MeSH synonyms, all-fields
    /// disjunctions, …). Useful for debugging "why does this search
    /// return *that*?".
    pub querytranslation: String,
}
