use serde::Serialize;

use super::common::EutilsIdent;

/// Query-string shape for `GET efetch.fcgi`.
///
/// Two retrieval modes share this struct:
/// * **By ID list:** set `id` to a comma-joined PMID list, leave
///   `web_env` / `query_key` as `None`. Used for `efetch_abstract`.
/// * **By History cursor:** set `web_env` + `query_key` (from a prior
///   `esearch` with `usehistory=y`), and optional `retstart` / `retmax`
///   to slice. Used for `efetch_bulk`. NCBI allows up to 10,000 records
///   per call this way.
#[derive(Debug, Serialize)]
pub(crate) struct EfetchRequest {
    /// NCBI database (`pubmed`).
    pub db: String,

    /// Comma-joined list of record IDs. Omit when using History.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Response format. We pass `"xml"` and parse it via `super::xml`.
    pub retmode: &'static str,

    /// Record subset to return. We use `"abstract"` to keep the
    /// payload small (no full-text-link tables etc.).
    pub rettype: &'static str,

    /// History server environment id. Capitalized per NCBI's expected
    /// param name; serialized as `WebEnv`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "WebEnv")]
    pub web_env: Option<String>,

    /// History server query key (paired with `web_env`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_key: Option<u32>,

    /// 0-based offset into the History result set. Only meaningful when
    /// `web_env` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retstart: Option<u32>,

    /// Number of records to return. Only meaningful when `web_env` is
    /// set. NCBI ceiling per call: 10,000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retmax: Option<u32>,

    /// `tool` + `email` + optional `api_key`, flattened.
    #[serde(flatten)]
    pub ident: EutilsIdent,
}
