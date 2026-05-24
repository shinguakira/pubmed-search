//! NCBI E-utilities client.
//!
//! Layout:
//! * `client.rs`    — `Client` struct, constructor, shared params.
//! * `esearch.rs`   — `esearch.fcgi` call.
//! * `esummary.rs`  — `esummary.fcgi` call.
//! * `efetch.rs`    — `efetch.fcgi` call (returns XML; parsed via `xml.rs`).
//! * `xml.rs`       — streaming PubMed XML parser.
//! * `dto/`         — response data structures (one file per concept).

mod client;
pub mod dto;
mod efetch;
mod esearch;
mod esummary;
mod xml;

pub use client::Client;
pub use dto::{ArticleDetail, Author, EsearchResult, Summary};
