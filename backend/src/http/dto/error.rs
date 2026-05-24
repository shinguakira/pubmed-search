use serde::Serialize;
use utoipa::ToSchema;

/// Wire-format error body returned for any non-2xx response.
///
/// The shape is intentionally tiny so it's easy to consume in JS:
/// `(await res.json()).message`. Pair it with the HTTP status code
/// for full context.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Short machine-readable code suitable for switch-on logic.
    /// Stable strings: `not_found`, `bad_request`, `internal`.
    pub code: &'static str,

    /// Human-readable explanation. Safe to surface directly to the
    /// end user — never includes stack traces or secrets.
    pub message: String,
}
