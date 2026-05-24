//! `GET /api/mesh` — MeSH term autocomplete via esearch + esummary on
//! the `mesh` db.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::error::{AppError, ErrorResponse};
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct MeshQuery {
    pub term: String,
    #[serde(default = "default_mesh_limit")]
    pub limit: u32,
}
fn default_mesh_limit() -> u32 {
    10
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeshResponse {
    pub count: u32,
    pub terms: Vec<MeshTerm>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeshTerm {
    pub id: String,
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/mesh",
    tag = "mesh",
    params(MeshQuery),
    responses(
        (status = 200, description = "MeSH term suggestions for a free-text query",
            body = MeshResponse),
        (status = 500, description = "Upstream NCBI error", body = ErrorResponse),
    ),
)]
pub async fn mesh_suggest(
    State(state): State<AppState>,
    Query(q): Query<MeshQuery>,
) -> Result<Json<MeshResponse>, AppError> {
    let es = state.ncbi.esearch("mesh", &q.term, 0, q.limit, None).await?;
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=mesh&id={}&retmode=json",
        es.ids.join(",")
    );
    let terms = if es.ids.is_empty() {
        vec![]
    } else {
        let body: serde_json::Value = reqwest::get(&url)
            .await
            .map_err(anyhow::Error::from)?
            .json()
            .await
            .map_err(anyhow::Error::from)?;
        let result = &body["result"];
        es.ids
            .iter()
            .map(|id| {
                let name = result[id]["ds_meshterms"]
                    .as_array()
                    .and_then(|a| a.first().and_then(|v| v.as_str()))
                    .or_else(|| result[id]["ds_meshui"].as_str())
                    .unwrap_or("")
                    .to_string();
                MeshTerm {
                    id: id.clone(),
                    name,
                }
            })
            .collect()
    };
    Ok(Json(MeshResponse {
        count: es.count,
        terms,
    }))
}
