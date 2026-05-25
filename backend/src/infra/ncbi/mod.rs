//! NCBI E-utilities client.
//!
//! Layout:
//! * `client.rs`    — `Client` struct, constructor, ident builder.
//! * `dto/`         — request + response DTOs at the NCBI ↔ backend
//!                    boundary.
//! * `esearch.rs`   — `esearch.fcgi` call.
//! * `esummary.rs`  — `esummary.fcgi` call.
//! * `efetch.rs`    — `efetch.fcgi` call (XML; parsed via `xml.rs`).
//! * `xml.rs`       — streaming PubMed XML parser.

mod client;
pub mod dto;
mod efetch;
mod esearch;
mod esummary;
mod xml;

pub use client::Client;
pub use dto::response::{ArticleDetail, Author, EsearchResult, Reference, Summary};
