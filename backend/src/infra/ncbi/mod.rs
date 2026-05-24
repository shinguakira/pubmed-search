//! NCBI E-utilities client.
//!
//! Layout:
//! * `client.rs`    — `Client` struct, constructor, shared params.
//! * `esearch.rs`   — `esearch.fcgi` call.
//! * `esummary.rs`  — `esummary.fcgi` call.
//! * `efetch.rs`    — `efetch.fcgi` call (returns XML; parsed via `xml.rs`).
//! * `xml.rs`       — streaming PubMed XML parser.
//! * `types/`       — parsed shapes of upstream responses (not DTOs —
//!                    no request body is sent to NCBI; query params only).

mod client;
mod efetch;
mod esearch;
mod esummary;
pub mod types;
mod xml;

pub use client::Client;
pub use types::{ArticleDetail, Author, EsearchResult, Summary};
