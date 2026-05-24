use serde::Serialize;

use super::common::EutilsIdent;

/// Query-string shape for `GET efetch.fcgi`.
#[derive(Debug, Serialize)]
pub(crate) struct EfetchRequest {
    /// NCBI database (`pubmed`).
    pub db: String,

    /// Comma-joined list of record IDs. For our use case we only ever
    /// fetch one PMID at a time, but NCBI accepts multiples.
    pub id: String,

    /// Response format. We pass `"xml"` and parse it via `super::xml`.
    pub retmode: &'static str,

    /// Record subset to return. We use `"abstract"` to keep the
    /// payload small (no full-text-link tables etc.).
    pub rettype: &'static str,

    /// `tool` + `email` + optional `api_key`, flattened.
    #[serde(flatten)]
    pub ident: EutilsIdent,
}
