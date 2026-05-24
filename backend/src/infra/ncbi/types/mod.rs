//! Parsed / normalized shapes of NCBI E-utilities output.
//!
//! These are **not** DTOs in the strict sense — we never send them to
//! NCBI (the upstream takes free-form HTTP query params, not bodies),
//! and what we receive is raw JSON / XML that this module's parsers
//! reshape into typed structs.
//!
//! Convention here:
//! * `summary.rs`  — output of `esummary.fcgi` per PMID.
//! * `article.rs`  — output of `efetch.fcgi` (XML parsed into structs).
//! * `esearch.rs`  — output of `esearch.fcgi`.
//!
//! These structs currently double as HTTP response bodies — they're
//! re-serialized as-is at the boundary. If the public API ever needs to
//! diverge from NCBI's shape, copy each into `http::dto::<resource>`
//! and translate at the handler.

pub mod article;
pub mod esearch;
pub mod summary;

pub use article::{ArticleDetail, Author};
pub use esearch::EsearchResult;
pub use summary::Summary;
