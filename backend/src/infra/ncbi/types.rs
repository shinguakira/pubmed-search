//! Response data structures shared between NCBI parsing and the HTTP
//! layer. For this PoC the same struct is used at both boundaries; if
//! the API ever drifts from NCBI's shape, split into NCBI-side and
//! HTTP-side types (move the HTTP DTO into `crate::http`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct Summary {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub source: String,
    pub pubdate: String,
    pub epubdate: String,
    pub volume: String,
    pub issue: String,
    pub pages: String,
    pub doi: String,
    pub pubtypes: Vec<String>,
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArticleDetail {
    pub pmid: String,
    pub title: String,
    pub abstract_text: String,
    pub authors: Vec<Author>,
    pub journal: String,
    pub pubdate: String,
    pub doi: String,
    pub keywords: Vec<String>,
    pub mesh_terms: Vec<String>,
    pub pubtypes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Author {
    pub last_name: String,
    pub fore_name: String,
    pub affiliation: String,
}
