//! Wire-format types exposed at the HTTP boundary. One file per resource.
//!
//! Convention: file names are the resource name (`search.rs`, not
//! `search_dto.rs`). Belonging to `dto/` is what marks them.
//!
//! Every `pub` field carries a `///` doc comment — utoipa picks these up
//! into the OpenAPI `description` for that property.

pub mod cite;
pub mod error;
pub mod export;
pub mod mesh;
pub mod search;
