//! Response DTOs — what NCBI sends back, after our parsers normalize
//! the raw JSON/XML into typed Rust.
//!
//! These structs are also re-exported from `crate::infra::ncbi` and
//! consumed directly as HTTP response bodies. If the public API ever
//! needs to diverge from NCBI's shape, copy each into
//! `crate::http::dto::<resource>` and translate at the handler.

pub mod article;
pub mod esearch;
pub mod summary;

pub use article::{ArticleDetail, Author, Reference};
pub use esearch::EsearchResult;
pub use summary::Summary;
