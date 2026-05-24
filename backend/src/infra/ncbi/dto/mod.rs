//! NCBI E-utilities DTOs.
//!
//! DTO = "data transfer object" — anything crossing a process / service
//! boundary. The boundary here is **our backend ↔ NCBI**, distinct from
//! the HTTP DTOs in `crate::http::dto` which cross **our backend ↔ the
//! browser**. Same concept, different boundary.
//!
//! Layout:
//! * `request/`  — Serialize-able structs whose fields become NCBI's
//!                 query-string parameters. One per upstream endpoint.
//! * `response/` — Deserialize-able / parsed structs that mirror what
//!                 NCBI returns. JSON (esearch / esummary) or XML
//!                 (efetch); both reshape into typed Rust here.

pub mod request;
pub mod response;
