//! Integration tests for the HTTP controllers — NO MOCKS.
//!
//! Each test spawns the real `axum` app on an ephemeral port, lets it call the
//! actual NCBI E-utilities, and asserts on the response shape plus measures
//! wall-clock processing time. Skipped automatically when the network is
//! unreachable so they don't break offline cargo test runs.
//!
//! Run with: `cargo test --manifest-path backend/Cargo.toml -- --nocapture`

use pubmed_backend::app;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Serialize NCBI traffic across tests so we don't trip the 3 req/s
/// anonymous rate limit when cargo runs tests in parallel.
fn ncbi_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

async fn spawn() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr: SocketAddr = listener.local_addr().expect("local_addr");
    tokio::spawn(async move {
        axum::serve(listener, app()).await.expect("server");
    });
    format!("http://{addr}")
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap()
}

async fn ncbi_reachable() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    c.head("https://eutils.ncbi.nlm.nih.gov/").send().await.is_ok()
}

macro_rules! require_network {
    () => {
        if !ncbi_reachable().await {
            eprintln!("[skip] NCBI not reachable");
            return;
        }
    };
}

#[tokio::test]
async fn search_returns_results_and_elapsed_ms() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    // Stay below NCBI's 3 req/s anonymous limit.
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();

    let wall = Instant::now();
    let res = client
        .get(format!("{base}/api/search?term=crispr&page=1&page_size=5"))
        .send()
        .await
        .expect("request");
    let total_ms = wall.elapsed().as_millis();
    assert_eq!(res.status(), 200, "non-200 from /api/search");

    let body: Value = res.json().await.expect("json body");
    let count = body["count"].as_u64().expect("count");
    let elapsed_ms = body["elapsed_ms"].as_u64().expect("elapsed_ms");
    let q = body["query_translation"].as_str().unwrap_or("");
    let results = body["results"].as_array().expect("results array");

    eprintln!(
        "[search] total_ms={total_ms} backend_elapsed_ms={elapsed_ms} count={count} results={}",
        results.len()
    );

    assert!(count > 1_000, "expected many results for 'crispr', got {count}");
    assert!(elapsed_ms > 0, "elapsed_ms should be measured");
    assert!(
        (elapsed_ms as u128) <= total_ms,
        "backend elapsed ({elapsed_ms}ms) cannot exceed wall time ({total_ms}ms)"
    );
    assert!(!q.is_empty(), "query_translation should be present");
    assert_eq!(results.len(), 5, "should return requested page size");

    let first = &results[0];
    assert!(first["pmid"].is_string());
    assert!(first["title"].is_string());
}

#[tokio::test]
async fn search_with_filters_changes_count() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    // Stay below NCBI's 3 req/s anonymous limit.
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();

    let unfiltered: Value = client
        .get(format!("{base}/api/search?term=crispr&page=1&page_size=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let started = Instant::now();
    let filtered: Value = client
        .get(format!(
            "{base}/api/search?term=crispr&page=1&page_size=1&filters=Review%5Bpt%5D"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ms = started.elapsed().as_millis();

    let unfiltered_count = unfiltered["count"].as_u64().unwrap();
    let filtered_count = filtered["count"].as_u64().unwrap();
    eprintln!(
        "[filter] unfiltered={unfiltered_count} review_only={filtered_count} wall_ms={ms}"
    );

    assert!(
        filtered_count > 0 && filtered_count < unfiltered_count,
        "Review filter should narrow but not eliminate results"
    );
}

#[tokio::test]
async fn article_returns_abstract_for_known_pmid() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    // Stay below NCBI's 3 req/s anonymous limit.
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();

    // Watson & Crick 1953 — has existed since pubmed existed.
    let pmid = "13054692";

    let started = Instant::now();
    let res = client
        .get(format!("{base}/api/article/{pmid}"))
        .send()
        .await
        .unwrap();
    let ms = started.elapsed().as_millis();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    eprintln!(
        "[article] pmid={pmid} wall_ms={ms} title_len={}",
        body["title"].as_str().unwrap_or("").len()
    );

    assert_eq!(body["pmid"].as_str().unwrap(), pmid);
    assert!(!body["title"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn search_bulk_returns_details_and_summary() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();

    let started = Instant::now();
    let res = client
        .get(format!(
            "{base}/api/search?term=crispr+cas9&page_size=10&bulk=true"
        ))
        .send()
        .await
        .expect("request");
    let total_ms = wall_ms(started);
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.expect("json");

    let results = body["results"].as_array().expect("results array");
    let details = body["details"].as_array().expect("details present when bulk=true");
    let elapsed_ms = body["elapsed_ms"].as_u64().unwrap();

    eprintln!(
        "[search bulk=true] total_ms={total_ms} backend_elapsed_ms={elapsed_ms} \
         results={} details={}",
        results.len(),
        details.len()
    );

    assert_eq!(results.len(), 10);
    assert_eq!(details.len(), 10, "details length must match results");

    // Same PMID in same position.
    for i in 0..results.len() {
        assert_eq!(
            results[i]["pmid"], details[i]["pmid"],
            "pmid mismatch at position {i}"
        );
    }
    // details carries the abstract (key thing the default path lacks).
    assert!(details[0]["abstract_text"].is_string());
}

#[tokio::test]
async fn search_default_omits_details() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();

    let body: Value = client
        .get(format!("{base}/api/search?term=crispr&page_size=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        body.get("details").map(|v| v.is_null()).unwrap_or(true),
        "details must be absent (or null) when bulk is not set; got {:?}",
        body.get("details")
    );
}

fn wall_ms(t: Instant) -> u128 {
    t.elapsed().as_millis()
}

#[tokio::test]
async fn cite_returns_all_formats() {
    let _gate = ncbi_gate().lock().await;
    require_network!();
    // Stay below NCBI's 3 req/s anonymous limit.
    tokio::time::sleep(Duration::from_millis(400)).await;
    let base = spawn().await;
    let client = http_client();
    let pmid = "13054692";

    let started = Instant::now();
    let body: Value = client
        .get(format!("{base}/api/cite/{pmid}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ms = started.elapsed().as_millis();
    eprintln!("[cite] pmid={pmid} wall_ms={ms}");

    for key in ["ama", "apa", "mla", "nlm", "bibtex"] {
        let v = body[key].as_str().unwrap_or("");
        assert!(!v.is_empty(), "cite format `{key}` should not be empty");
    }
    assert!(body["bibtex"].as_str().unwrap().contains(&format!("pmid{pmid}")));
}
