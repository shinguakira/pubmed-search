//! `GET /api/cite/{pmid}` — generate 5 citation formats from a single
//! efetch call.

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{AppError, ErrorResponse};
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct CiteResponse {
    pub ama: String,
    pub apa: String,
    pub mla: String,
    pub nlm: String,
    pub bibtex: String,
}

#[utoipa::path(
    get,
    path = "/api/cite/{pmid}",
    tag = "pubmed",
    params(("pmid" = String, Path, description = "PubMed ID")),
    responses(
        (status = 200, description = "Citation strings in 5 common formats",
            body = CiteResponse),
        (status = 500, description = "Upstream NCBI error", body = ErrorResponse),
    ),
)]
pub async fn cite(
    State(state): State<AppState>,
    Path(pmid): Path<String>,
) -> Result<Json<CiteResponse>, AppError> {
    let d = state.ncbi.efetch_abstract(&pmid).await?;

    let authors_short = d
        .authors
        .iter()
        .map(|a| {
            let initials: String = a
                .fore_name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect();
            format!("{} {}", a.last_name, initials)
        })
        .collect::<Vec<_>>();
    let authors_apa = d
        .authors
        .iter()
        .map(|a| {
            let initials: String = a
                .fore_name
                .split_whitespace()
                .filter_map(|w| w.chars().next().map(|c| format!("{}.", c)))
                .collect();
            format!("{}, {}", a.last_name, initials)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let first_author = d
        .authors
        .first()
        .map(|a| format!("{}, {}", a.last_name, a.fore_name))
        .unwrap_or_default();

    let year = d
        .pubdate
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();

    let ama = format!(
        "{}. {} {}. {};{}. PMID: {}{}",
        authors_short.join(", "),
        d.title,
        d.journal,
        year,
        "",
        d.pmid,
        if d.doi.is_empty() {
            String::new()
        } else {
            format!(" doi:{}", d.doi)
        }
    );
    let apa = format!(
        "{} ({}). {}. {}.{}",
        authors_apa,
        year,
        d.title,
        d.journal,
        if d.doi.is_empty() {
            String::new()
        } else {
            format!(" https://doi.org/{}", d.doi)
        }
    );
    let mla = format!(
        "{} \"{}\" {}, {}.{}",
        first_author,
        d.title,
        d.journal,
        year,
        if d.doi.is_empty() {
            String::new()
        } else {
            format!(" doi:{}.", d.doi)
        }
    );
    let nlm = ama.clone();
    let bibtex = format!(
        "@article{{pmid{pmid},\n  title   = {{ {title} }},\n  author  = {{ {authors} }},\n  journal = {{ {journal} }},\n  year    = {{ {year} }},\n  doi     = {{ {doi} }},\n  pmid    = {{ {pmid} }}\n}}",
        pmid = d.pmid,
        title = d.title,
        authors = d
            .authors
            .iter()
            .map(|a| format!("{}, {}", a.last_name, a.fore_name))
            .collect::<Vec<_>>()
            .join(" and "),
        journal = d.journal,
        year = year,
        doi = d.doi,
    );

    Ok(Json(CiteResponse {
        ama,
        apa,
        mla,
        nlm,
        bibtex,
    }))
}
