//! Shared application state injected into every handler via Axum's
//! `State<AppState>` extractor.
//!
//! Grow this struct as the app picks up new dependencies (DB pool,
//! Redis client, feature-flag store, …). Keep it `Clone` (each inner
//! type should be cheap to clone — wrap heavy things in `Arc`).

use crate::infra::ncbi;

#[derive(Clone)]
pub struct AppState {
    pub ncbi: ncbi::Client,
    // Future additions:
    // pub db: sqlx::PgPool,
    // pub config: std::sync::Arc<Config>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ncbi: ncbi::Client::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
