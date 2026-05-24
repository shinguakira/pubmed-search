use serde::Serialize;

use super::common::EutilsIdent;

/// Query-string shape for `GET esearch.fcgi`.
#[derive(Debug, Serialize)]
pub(crate) struct EsearchRequest {
    /// NCBI database to search (`pubmed`, `mesh`, …).
    pub db: String,

    /// Query string in PubMed grammar (field tags, booleans).
    pub term: String,

    /// 0-based offset into the matching result set.
    pub retstart: u32,

    /// Number of records to return starting at `retstart`.
    pub retmax: u32,

    /// Response format. We always pass `"json"` for this endpoint.
    pub retmode: &'static str,

    /// Sort order keyword. Omitted from URL when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    /// `"y"` to make NCBI stash the matching IDs on the History server
    /// and return `WebEnv` + `QueryKey` we can reuse for bulk efetch.
    /// Omitted from URL when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usehistory: Option<&'static str>,

    /// `tool` + `email` + optional `api_key`, flattened into the URL.
    #[serde(flatten)]
    pub ident: EutilsIdent,
}
