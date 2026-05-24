use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use utoipa::{IntoParams, ToSchema};

use crate::pubmed::{ArticleDetail, Client, Summary};

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Free-text PubMed query, e.g. `crispr cas9` or `covid 2024[dp]`.
    pub term: String,
    /// 1-based page number.
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Sort order. One of: `relevance`, `pub_date`, `first_author`, `journal`, `title`.
    pub sort: Option<String>,
    /// Comma-separated PubMed filter expressions appended to the term,
    /// e.g. `humans[Filter],english[lang],2020:2025[dp],Review[pt]`.
    pub filters: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub count: u32,
    pub page: u32,
    pub page_size: u32,
    pub query_translation: String,
    /// Server-side wall-clock duration for the upstream NCBI roundtrip,
    /// in milliseconds.
    pub elapsed_ms: u64,
    pub results: Vec<Summary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

#[utoipa::path(
    get,
    path = "/api/search",
    tag = "pubmed",
    params(SearchQuery),
    responses(
        (status = 200, description = "Paginated PubMed search results",
            body = SearchResponse),
        (status = 500, description = "Upstream NCBI error", body = ErrorBody),
    ),
)]
pub async fn search(
    State(client): State<Client>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let started = Instant::now();
    let mut term = q.term.trim().to_string();
    if let Some(f) = q.filters.as_ref() {
        for filt in f.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            term = format!("({term}) AND {filt}");
        }
    }
    let retstart = q.page.saturating_sub(1) * q.page_size;
    let es = client
        .esearch("pubmed", &term, retstart, q.page_size, q.sort.as_deref())
        .await?;
    let summaries = client.esummary("pubmed", &es.ids).await?;
    Ok(Json(SearchResponse {
        count: es.count,
        page: q.page,
        page_size: q.page_size,
        query_translation: es.querytranslation,
        elapsed_ms: started.elapsed().as_millis() as u64,
        results: summaries,
    }))
}

#[utoipa::path(
    get,
    path = "/api/article/{pmid}",
    tag = "pubmed",
    params(("pmid" = String, Path, description = "PubMed ID")),
    responses(
        (status = 200, description = "Structured article detail (abstract, authors, MeSH, …)",
            body = ArticleDetail),
        (status = 500, description = "Upstream NCBI error", body = ErrorBody),
    ),
)]
pub async fn article(
    State(client): State<Client>,
    Path(pmid): Path<String>,
) -> Result<Json<ArticleDetail>, ApiError> {
    let detail = client.efetch_abstract(&pmid).await?;
    Ok(Json(detail))
}

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
        (status = 500, description = "Upstream NCBI error", body = ErrorBody),
    ),
)]
pub async fn mesh_suggest(
    State(client): State<Client>,
    Query(q): Query<MeshQuery>,
) -> Result<Json<MeshResponse>, ApiError> {
    let es = client.esearch("mesh", &q.term, 0, q.limit, None).await?;
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=mesh&id={}&retmode=json",
        es.ids.join(",")
    );
    let terms = if es.ids.is_empty() {
        vec![]
    } else {
        let body: serde_json::Value = reqwest::get(&url).await?.json().await?;
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
                MeshTerm { id: id.clone(), name }
            })
            .collect()
    };
    Ok(Json(MeshResponse { count: es.count, terms }))
}

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
        (status = 500, description = "Upstream NCBI error", body = ErrorBody),
    ),
)]
pub async fn cite(
    State(client): State<Client>,
    Path(pmid): Path<String>,
) -> Result<Json<CiteResponse>, ApiError> {
    let d = client.efetch_abstract(&pmid).await?;
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
    let first_author = d.authors.first().map(|a| {
        format!("{}, {}", a.last_name, a.fore_name)
    }).unwrap_or_default();

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
        if d.doi.is_empty() { String::new() } else { format!(" doi:{}", d.doi) }
    );
    let apa = format!(
        "{} ({}). {}. {}.{}",
        authors_apa,
        year,
        d.title,
        d.journal,
        if d.doi.is_empty() { String::new() } else { format!(" https://doi.org/{}", d.doi) }
    );
    let mla = format!(
        "{} \"{}\" {}, {}.{}",
        first_author,
        d.title,
        d.journal,
        year,
        if d.doi.is_empty() { String::new() } else { format!(" doi:{}.", d.doi) }
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

    Ok(Json(CiteResponse { ama, apa, mla, nlm, bibtex }))
}

pub struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        ApiError(e.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("api error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody { error: self.0.to_string() }),
        )
            .into_response()
    }
}
