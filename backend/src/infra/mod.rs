//! Infrastructure / IO boundary. Everything that talks to the outside
//! world lives here: HTTP clients for external APIs, DB pools, file IO,
//! message queues, etc.
//!
//! Add a sibling module per IO concern (`ncbi`, `db`, `cache`, …).

pub mod cache;
pub mod ncbi;
