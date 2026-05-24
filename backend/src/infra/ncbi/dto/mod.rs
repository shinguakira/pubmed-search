//! Response shapes parsed out of NCBI E-utilities.
//!
//! One file per upstream concept. These types currently double as the
//! HTTP response bodies (re-exported through `http::dto`). If the public
//! API ever needs to diverge from NCBI's shape, copy each struct into
//! `http::dto::<resource>` and translate at the handler boundary.
//!
//! Every `pub` field carries a `///` doc comment — utoipa picks these
//! up into the OpenAPI `description` for that property.

pub mod article;
pub mod esearch;
pub mod summary;

pub use article::{ArticleDetail, Author};
pub use esearch::EsearchResult;
pub use summary::Summary;
