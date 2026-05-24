use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

/// Output format for `GET /api/search/export`.
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// BibTeX. One `@article` per record. `Content-Type:
    /// application/x-bibtex`.
    Bibtex,
    /// CSV with header row. `Content-Type: text/csv`.
    Csv,
    /// JSON array of `ArticleDetail`. `Content-Type:
    /// application/json`.
    Json,
}

/// Query parameters for `GET /api/search/export` — same search grammar
/// as `/api/search`, plus an output `format` and a hard cap (`max`).
#[derive(Debug, Deserialize, IntoParams)]
pub struct ExportQuery {
    /// Free-text PubMed query (same grammar as `/api/search`).
    pub term: String,

    /// Comma-separated raw PubMed filter expressions (same shape as
    /// `/api/search`).
    pub filters: Option<String>,

    /// Sort order. Same options as `/api/search`.
    pub sort: Option<String>,

    /// Maximum number of records to bundle into the export. NCBI's
    /// per-call ceiling for `efetch` via History is 10,000; we clamp
    /// to that. Default: 200.
    #[serde(default = "default_max")]
    pub max: u32,

    /// Output format. One of: `bibtex`, `csv`, `json`.
    pub format: ExportFormat,
}

fn default_max() -> u32 {
    200
}
