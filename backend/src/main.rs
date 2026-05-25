//! Binary entry point. Keep this file tiny on purpose — the actual router
//! lives in `lib.rs` so integration tests (`tests/api.rs`) and the OpenAPI
//! generator (`src/bin/gen-openapi.rs`) can reuse the exact same wiring.

use pubmed_backend::app;
use std::net::SocketAddr;

// `#[tokio::main]` rewrites this `fn main` into one that spins up a Tokio
// async runtime, then runs the body on it. Without it, you cannot `.await`.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Structured logging. `RUST_LOG=...` overrides the default filter at
    // runtime, otherwise we log our own crate at INFO.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pubmed_backend=info,tower_http=info".into()),
        )
        .init();

    // Two modes:
    //   * dev — bind 127.0.0.1:8787 so Vite proxy / Playwright stay happy.
    //   * container deploy — Azure Web App, Cloud Run and Fly all inject
    //     `$PORT`; the container must listen on 0.0.0.0 to be reachable
    //     from the platform's frontend.
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8787);
    let host = if std::env::var("PORT").is_ok() { "0.0.0.0" } else { "127.0.0.1" };
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // `app()` returns the fully assembled `axum::Router` (search/article/
    // mesh/cite + Swagger UI + CORS). `axum::serve` drives the listener
    // until the process is stopped.
    axum::serve(listener, app()).await?;
    Ok(())
}
