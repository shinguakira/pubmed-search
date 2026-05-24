//! NCBI E-utilities client.
//!
//! Public surface:
//! * [`Client`] — the HTTP client wrapper, cloned per request via Axum's
//!   `State`.
//! * [`Summary`], [`ArticleDetail`], [`Author`] — response DTOs shared
//!   with the HTTP layer (re-exported in `http/*` handlers as response
//!   bodies and OpenAPI schemas).
//! * [`EsearchResult`] — internal result of the `esearch` call;
//!   re-exported in case downstream code wants the raw IDs.
//!
//! The module is split by upstream endpoint:
//! * `client.rs`    — Client struct, constructor, shared params.
//! * `esearch.rs`   — `esearch.fcgi` call + `EsearchResult` type.
//! * `esummary.rs`  — `esummary.fcgi` call.
//! * `efetch.rs`    — `efetch.fcgi` call (returns XML; parsed via `xml.rs`).
//! * `xml.rs`       — streaming PubMed XML parser.
//! * `types.rs`     — response data structures.

mod client;
mod efetch;
mod esearch;
mod esummary;
mod types;
mod xml;

pub use client::Client;
pub use esearch::EsearchResult;
pub use types::{ArticleDetail, Author, Summary};
