//! Side-by-side benchmark of fetching N PubMed records two ways:
//!   * INDIV — N separate `efetch_abstract` calls (the old per-PMID path)
//!   * BULK  — 1 `esearch(usehistory=y)` + 1 `efetch_bulk(WebEnv, ...)`
//!
//! Hits LIVE NCBI for both paths so the numbers reflect reality. Tests
//! are gated behind the shared `ncbi_gate` mutex and throttled to stay
//! under NCBI's 3 req/s anonymous limit; total benchmark wall time is
//! dominated by the individual path.
//!
//! Run with: `cargo test --test benchmark -- --nocapture`

use pubmed_backend::infra::ncbi::Client;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

fn ncbi_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

/// Inter-request delay used for the INDIV path. NCBI's anonymous limit
/// is 3 req/s = 333 ms; we add a small margin.
const INDIV_THROTTLE: Duration = Duration::from_millis(380);

async fn ncbi_reachable() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    c.head("https://eutils.ncbi.nlm.nih.gov/").send().await.is_ok()
}

/// Fetch N article details one-by-one, returning the wall-clock duration.
async fn bench_indiv(client: &Client, ids: &[String]) -> Duration {
    let started = Instant::now();
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(INDIV_THROTTLE).await;
        }
        let _ = client
            .efetch_abstract(id)
            .await
            .expect("efetch_abstract failed");
    }
    started.elapsed()
}

/// Fetch the first N articles for `term` via History server in one
/// efetch_bulk call. Returns wall-clock duration including the prior
/// esearch.
async fn bench_bulk(client: &Client, term: &str, n: u32) -> Duration {
    let started = Instant::now();
    let es = client
        .esearch("pubmed", term, 0, n, None, true)
        .await
        .expect("esearch failed");
    let web_env = es.web_env.expect("no WebEnv");
    let query_key = es.query_key.expect("no QueryKey");
    let _articles = client
        .efetch_bulk(&web_env, query_key, 0, n)
        .await
        .expect("efetch_bulk failed");
    started.elapsed()
}

#[tokio::test]
async fn benchmark_indiv_vs_bulk_across_scenarios() {
    // Hold the gate for the entire benchmark — we don't want other live
    // tests interleaving NCBI traffic mid-measurement.
    let _gate = ncbi_gate().lock().await;
    if !ncbi_reachable().await {
        eprintln!("[skip] NCBI not reachable");
        return;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    // (term, N) — kept small enough that INDIV stays under ~30s each.
    let scenarios: &[(&str, u32)] = &[
        ("crispr cas9", 10),
        ("covid", 20),
        ("alzheimer review", 30),
        ("crispr cas9", 50),
    ];

    eprintln!(
        "\n{:─^72}\n{:<22} {:>5} {:>11} {:>11} {:>9} {:>9}\n{:─^72}",
        " BENCH: indiv vs bulk ",
        "term",
        "N",
        "indiv (s)",
        "bulk (s)",
        "saved",
        "speedup",
        ""
    );

    for &(term, n) in scenarios {
        // Step 1: collect N PMIDs to drive the INDIV path with.
        let es = client
            .esearch("pubmed", term, 0, n, None, false)
            .await
            .expect("esearch failed");
        let ids: Vec<String> = es.ids.iter().take(n as usize).cloned().collect();
        // Small inter-scenario rest so we don't burst NCBI.
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Step 2: time INDIV (slow path).
        let indiv = bench_indiv(&client, &ids).await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Step 3: time BULK (fast path).
        let bulk = bench_bulk(&client, term, n).await;

        let saved = indiv.checked_sub(bulk).unwrap_or_default();
        let speedup = indiv.as_secs_f64() / bulk.as_secs_f64();
        eprintln!(
            "{:<22} {:>5} {:>11.2} {:>11.2} {:>8.2}s {:>8.1}x",
            truncate(term, 22),
            n,
            indiv.as_secs_f64(),
            bulk.as_secs_f64(),
            saved.as_secs_f64(),
            speedup
        );

        // Sanity: bulk must be strictly faster than indiv for any N>1.
        assert!(
            bulk < indiv,
            "bulk ({bulk:?}) should be faster than indiv ({indiv:?}) for N={n}"
        );

        // Polite inter-scenario delay.
        tokio::time::sleep(Duration::from_millis(800)).await;
    }
    eprintln!("{:─^72}", "");
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut t = s[..n.saturating_sub(1)].to_string();
        t.push('…');
        t
    }
}
