//! End-to-end benchmark — full-corpus fetch.
//!
//! Surfaces a **structural** difference between the two access
//! patterns that wall-time alone doesn't show:
//!
//! * **DEFAULT** — for each 10k slice, re-issue `esearch(retstart=k)`
//!   then `POST efetch.fcgi` with the page's id list.
//!   ⇒ NCBI hard-caps `esearch retstart ≤ 9998` for PubMed; this
//!   path **physically cannot reach record 10,000 or beyond**.
//!   The first slice (retstart=0) succeeds; the second fails with
//!   `Search Backend failed: 'retstart' cannot be larger than 9998`.
//! * **BULK**    — `esearch(usehistory=y)` once, then `POST
//!   efetch.fcgi` with `(WebEnv, query_key, retstart=k, retmax=10000)`
//!   per slice. The History server has no `retstart` cap, so we can
//!   walk the full corpus.
//!
//! For terms whose corpus ≤ 9999 both paths finish; for anything
//! bigger only bulk completes.
//!
//! Run with: `cargo test --test benchmark -- --ignored --nocapture`

use pubmed_backend::infra::ncbi::Client;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

fn ncbi_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

const PAGE: u32 = 10_000;
const THROTTLE: Duration = Duration::from_millis(350);

async fn ncbi_reachable() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    c.head("https://eutils.ncbi.nlm.nih.gov/").send().await.is_ok()
}

struct RunStats {
    dur: Duration,
    got: usize,
    calls: u32,
    error: Option<String>,
}

/// DEFAULT: re-esearch per slice + POST efetch by id list.
/// Stops (and records the error) the moment NCBI rejects `retstart`.
async fn bench_default(client: &Client, term: &str, total: u32) -> RunStats {
    let started = Instant::now();
    let mut got = 0usize;
    let mut calls = 0u32;
    let mut retstart = 0u32;
    while retstart < total {
        let es_result = client
            .esearch("pubmed", term, retstart, PAGE, None, false)
            .await;
        calls += 1;
        let es = match es_result {
            Ok(es) => es,
            Err(e) => {
                return RunStats {
                    dur: started.elapsed(),
                    got,
                    calls,
                    error: Some(format!("esearch retstart={retstart}: {e}")),
                };
            }
        };
        tokio::time::sleep(THROTTLE).await;
        let records = client
            .efetch_by_ids(&es.ids)
            .await
            .expect("efetch_by_ids failed");
        calls += 1;
        got += records.len();
        retstart += PAGE;
        tokio::time::sleep(THROTTLE).await;
    }
    RunStats {
        dur: started.elapsed(),
        got,
        calls,
        error: None,
    }
}

/// BULK: one esearch(usehistory=y) + N POST efetch via WebEnv.
///
/// `retmax` on the esearch must be ≥ `total` (capped at NCBI's
/// 100,000 per-esearch limit) — otherwise the History server parks
/// only the first `retmax` records and subsequent `efetch` calls with
/// larger `retstart` return empty.
async fn bench_bulk(client: &Client, term: &str, total: u32) -> RunStats {
    let started = Instant::now();
    let park = total.min(100_000);
    let es = client
        .esearch("pubmed", term, 0, park, None, true)
        .await
        .expect("esearch (history) failed");
    let mut calls = 1u32;
    let web_env = es.web_env.expect("no WebEnv");
    let query_key = es.query_key.expect("no QueryKey");
    tokio::time::sleep(THROTTLE).await;

    let mut got = 0usize;
    let mut retstart = 0u32;
    while retstart < total {
        let records = client
            .efetch_bulk(&web_env, query_key, retstart, PAGE)
            .await
            .expect("efetch_bulk failed");
        calls += 1;
        got += records.len();
        retstart += PAGE;
        tokio::time::sleep(THROTTLE).await;
    }
    RunStats {
        dur: started.elapsed(),
        got,
        calls,
        error: None,
    }
}

#[tokio::test]
#[ignore = "full-fetch benchmark hits NCBI for thousands of calls; run with --ignored"]
async fn benchmark_full_fetch_default_vs_bulk() {
    let _gate = ncbi_gate().lock().await;
    if !ncbi_reachable().await {
        eprintln!("[skip] NCBI not reachable");
        return;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    // One term whose corpus is comfortably within the esearch
    // retstart cap (so default and bulk both finish), and one above
    // it (so the structural gap shows).
    let terms: &[&str] = &[
        "crispr cas9", // ~39k — default fails at retstart=10000
        "glaucoma",    // ~95k — same
    ];

    eprintln!(
        "\n{:─^108}\n{:<18} {:>8} {:>16} {:>16} {:>9} {:>9} {:>9}\n{:─^108}",
        " BENCH: full corpus fetch — default vs bulk ",
        "term",
        "hits",
        "default got/s",
        "bulk got/s",
        "calls D",
        "calls B",
        "speedup",
        ""
    );

    for &term in terms {
        let probe = client
            .esearch("pubmed", term, 0, 0, None, false)
            .await
            .expect("esearch probe failed");
        let hits = probe.count;
        tokio::time::sleep(THROTTLE).await;

        let def = bench_default(&client, term, hits).await;
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let bulk = bench_bulk(&client, term, hits).await;

        let speedup = def.dur.as_secs_f64() / bulk.dur.as_secs_f64();
        let def_label = if def.error.is_some() {
            format!("{}/{:.0}s ✗", def.got, def.dur.as_secs_f64())
        } else {
            format!("{}/{:.0}s", def.got, def.dur.as_secs_f64())
        };
        let bulk_label = format!("{}/{:.0}s", bulk.got, bulk.dur.as_secs_f64());

        eprintln!(
            "{:<18} {:>8} {:>16} {:>16} {:>9} {:>9} {:>8.2}x",
            truncate(term, 18),
            human_count(hits),
            def_label,
            bulk_label,
            def.calls,
            bulk.calls,
            speedup
        );
        if let Some(err) = def.error {
            eprintln!("    default failed: {err}");
        }

        tokio::time::sleep(Duration::from_millis(2000)).await;
    }
    eprintln!("{:─^108}", "");
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

fn human_count(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
