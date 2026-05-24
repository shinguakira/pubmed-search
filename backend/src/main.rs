mod pubmed;
mod routes;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pubmed_backend=info,tower_http=info".into()),
        )
        .init();

    let client = pubmed::Client::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/search", get(routes::search))
        .route("/api/article/:pmid", get(routes::article))
        .route("/api/mesh", get(routes::mesh_suggest))
        .route("/api/cite/:pmid", get(routes::cite))
        .with_state(client)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = "0.0.0.0:8787".parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
