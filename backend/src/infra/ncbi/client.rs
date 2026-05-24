use super::dto::request::common::EutilsIdent;

/// Base URL for all NCBI E-utilities endpoints.
pub(super) const EUTILS: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";

/// HTTP client for NCBI E-utilities. `Clone` is cheap — `reqwest::Client`
/// shares its connection pool internally — so Axum can hand a copy to
/// each request via `State`.
#[derive(Clone)]
pub struct Client {
    pub(super) http: reqwest::Client,
    pub(super) api_key: Option<String>,
    pub(super) tool: String,
    pub(super) email: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("pubmed-search-poc/0.1")
                .build()
                .expect("reqwest client"),
            api_key: std::env::var("NCBI_API_KEY").ok(),
            tool: std::env::var("NCBI_TOOL").unwrap_or_else(|_| "pubmed-search-poc".into()),
            email: std::env::var("NCBI_EMAIL").unwrap_or_else(|_| "dev@example.com".into()),
        }
    }

    /// Build the per-request identifier block (`tool`/`email`/`api_key`)
    /// that NCBI asks every call to carry. Flattened into each request
    /// DTO via `#[serde(flatten)]`.
    pub(super) fn ident(&self) -> EutilsIdent {
        EutilsIdent {
            tool: self.tool.clone(),
            email: self.email.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
