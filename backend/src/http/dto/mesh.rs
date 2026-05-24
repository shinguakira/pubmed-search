use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for `GET /api/mesh` — MeSH term suggestions.
#[derive(Debug, Deserialize, IntoParams)]
pub struct MeshQuery {
    /// Free-text fragment to look up in the MeSH thesaurus.
    pub term: String,

    /// Max suggestions to return (default 10).
    #[serde(default = "default_mesh_limit")]
    pub limit: u32,
}
fn default_mesh_limit() -> u32 {
    10
}

/// Response body for `GET /api/mesh`.
#[derive(Debug, Serialize, ToSchema)]
pub struct MeshResponse {
    /// Total matching MeSH terms in NCBI's mesh db (not capped to `limit`).
    pub count: u32,

    /// Returned suggestions (length ≤ `limit`).
    pub terms: Vec<MeshTerm>,
}

/// A single MeSH descriptor.
#[derive(Debug, Serialize, ToSchema)]
pub struct MeshTerm {
    /// NCBI's MeSH unique identifier (UID). Stable across reindexes.
    pub id: String,

    /// Human-readable preferred term (e.g. `Aspirin`).
    pub name: String,
}
