use serde::Serialize;

use super::common::EutilsIdent;

/// Query-string shape for `GET esummary.fcgi`.
#[derive(Debug, Serialize)]
pub(crate) struct EsummaryRequest {
    /// NCBI database (`pubmed`, `mesh`, …).
    pub db: String,

    /// Comma-joined list of record IDs (e.g. `25315507,26470680`).
    pub id: String,

    /// Response format. We always pass `"json"` for this endpoint.
    pub retmode: &'static str,

    /// `tool` + `email` + optional `api_key`, flattened.
    #[serde(flatten)]
    pub ident: EutilsIdent,
}
