//! `GET /api/mesh` — MeSH term autocomplete via esearch + esummary on
//! the `mesh` db.

use axum::extract::{Query, State};
use axum::Json;

use crate::error::AppError;
use crate::http::dto::error::ErrorResponse;
use crate::http::dto::mesh::{MeshQuery, MeshResponse, MeshTerm};
use crate::state::AppState;

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
    let es = state.ncbi.esearch("mesh", &q.term, 0, q.limit, None, false).await?;
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
