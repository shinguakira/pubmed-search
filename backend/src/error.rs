//! Centralized error type for the HTTP layer.
//!
//! Pattern: each subsystem (infra/ncbi, future infra/db, domain) returns
//! its own typed error. `AppError` wraps them via `#[from]` so handlers
//! can use `?` freely. The single `IntoResponse` impl below maps every
//! variant to an HTTP status + JSON body, keeping per-handler code clean.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Wire-format error body returned to clients. Listed in the OpenAPI
/// schema as `ErrorResponse`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Short machine-readable error code (e.g. `not_found`, `internal`).
    pub code: &'static str,
    /// Human-readable explanation. Safe to surface to the user.
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// Wrapping any opaque error (anything `Into<anyhow::Error>`) — used
    /// while we still rely on `anyhow` inside the infra/ncbi calls.
    /// Specialize into typed variants as the codebase grows.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    /// Reserved for "resource exists but the requested item is missing".
    /// Not used yet — placeholder for the DB layer.
    #[error("not found")]
    #[allow(dead_code)]
    NotFound,

    /// Reserved for client input validation failures.
    /// Not used yet — placeholder for richer request validation.
    #[error("bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        // Only log at error level for unexpected (500) cases.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("app error: {:?}", self);
        }
        let body = ErrorResponse {
            code,
            message: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}
