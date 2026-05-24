use serde::Serialize;

/// Identifier params NCBI asks every request to carry.
///
/// Flattened into each endpoint-specific request struct via
/// `#[serde(flatten)]` so all three appear at the top level of the
/// generated query string.
#[derive(Debug, Serialize)]
pub(crate) struct EutilsIdent {
    /// Application identifier. NCBI uses this to contact you if your
    /// traffic is abusive; required by NCBI guidelines.
    pub tool: String,

    /// Contact email. Same purpose as `tool`. Required.
    pub email: String,

    /// Optional NCBI API key. Presence bumps the rate limit from
    /// 3 req/s → 10 req/s. Omitted from the query string when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}
