//! Request DTOs — typed query-string shapes for each NCBI endpoint.
//!
//! NCBI E-utilities are HTTP GET with query parameters (no request body).
//! Serializing one of these structs via `reqwest::RequestBuilder::query`
//! produces the URL we send.
//!
//! Visibility is `pub(crate)` — these are an implementation detail of
//! the NCBI client and not part of any external surface.

pub(crate) mod common;
pub(crate) mod efetch;
pub(crate) mod esearch;
pub(crate) mod esummary;
