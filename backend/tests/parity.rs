//! Parity tests: the bulk path (`esearch(usehistory) + efetch_bulk`)
//! must return the *same data* as the individual path
//! (`efetch_abstract`) for the *same* PMIDs.
//!
//! Field-by-field equality check on `ArticleDetail` — covers every
//! field on the struct, including `Author` sub-records. Any drift
//! between the two paths fails the test with a specific message
//! identifying which field disagreed.
//!
//! Live NCBI; gated + throttled like the other integration tests.

use pubmed_backend::infra::ncbi::{ArticleDetail, Author, Client};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Mutex;

fn ncbi_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

async fn ncbi_reachable() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    c.head("https://eutils.ncbi.nlm.nih.gov/").send().await.is_ok()
}

/// Compare every public field of two `ArticleDetail`s. Panics with a
/// useful message identifying the first divergent field.
fn assert_articles_equal(label: &str, a: &ArticleDetail, b: &ArticleDetail) {
    assert_eq!(a.pmid, b.pmid, "{label}: pmid differs");
    assert_eq!(a.title, b.title, "{label}: title differs");
    assert_eq!(
        a.abstract_text, b.abstract_text,
        "{label}: abstract_text differs"
    );
    assert_eq!(a.journal, b.journal, "{label}: journal differs");
    assert_eq!(a.pubdate, b.pubdate, "{label}: pubdate differs");
    assert_eq!(a.doi, b.doi, "{label}: doi differs");
    assert_eq!(a.keywords, b.keywords, "{label}: keywords differ");
    assert_eq!(a.mesh_terms, b.mesh_terms, "{label}: mesh_terms differ");
    assert_eq!(a.pubtypes, b.pubtypes, "{label}: pubtypes differ");
    assert_eq!(
        a.authors.len(),
        b.authors.len(),
        "{label}: author count differs ({} vs {})",
        a.authors.len(),
        b.authors.len()
    );
    for (i, (ai, bi)) in a.authors.iter().zip(b.authors.iter()).enumerate() {
        assert_authors_equal(&format!("{label}.authors[{i}]"), ai, bi);
    }
}

fn assert_authors_equal(label: &str, a: &Author, b: &Author) {
    assert_eq!(a.last_name, b.last_name, "{label}: last_name differs");
    assert_eq!(a.fore_name, b.fore_name, "{label}: fore_name differs");
    assert_eq!(a.affiliation, b.affiliation, "{label}: affiliation differs");
}

/// Single PMID — bulk fetch of a 1-PMID result set should equal
/// `efetch_abstract` for that same PMID.
#[tokio::test]
async fn bulk_and_individual_agree_for_one_pmid() {
    let _gate = ncbi_gate().lock().await;
    if !ncbi_reachable().await {
        eprintln!("[skip] NCBI not reachable");
        return;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;

    let client = Client::new();
    // Watson & Crick 1953 — stable record with a known structured abstract.
    let pmid = "13054692";

    // Path A: individual.
    let indiv = client.efetch_abstract(pmid).await.expect("efetch_abstract");
    tokio::time::sleep(Duration::from_millis(400)).await;

    // Path B: search for the exact PMID via History, then bulk-fetch.
    let term = format!("{pmid}[uid]");
    let es = client
        .esearch("pubmed", &term, 0, 1, None, true)
        .await
        .expect("esearch");
    let web_env = es.web_env.expect("no WebEnv");
    let query_key = es.query_key.expect("no QueryKey");
    let mut bulk = client
        .efetch_bulk(&web_env, query_key, 0, 1)
        .await
        .expect("efetch_bulk");
    assert_eq!(bulk.len(), 1, "expected exactly 1 record from bulk");
    let from_bulk = bulk.remove(0);

    eprintln!(
        "[parity 1-pmid] pmid={pmid} abstract_len={} authors={}",
        indiv.abstract_text.len(),
        indiv.authors.len()
    );
    assert_articles_equal("single", &indiv, &from_bulk);
}

/// Multiple PMIDs — bulk fetch of N PMIDs should agree with N
/// individual `efetch_abstract` calls, record by record.
#[tokio::test]
async fn bulk_and_individual_agree_for_many_pmids() {
    let _gate = ncbi_gate().lock().await;
    if !ncbi_reachable().await {
        eprintln!("[skip] NCBI not reachable");
        return;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;

    let client = Client::new();
    // 5 mixed CRISPR-related PMIDs.
    let pmids = ["25315507", "27699445", "26470680", "31727474", "29358495"];

    // Path A: individual, throttled.
    let mut indiv_records = Vec::with_capacity(pmids.len());
    for p in pmids {
        indiv_records.push(client.efetch_abstract(p).await.expect("efetch_abstract"));
        tokio::time::sleep(Duration::from_millis(380)).await;
    }

    // Path B: one esearch (joining with OR) + one efetch_bulk.
    let term = pmids
        .iter()
        .map(|p| format!("{p}[uid]"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let es = client
        .esearch("pubmed", &term, 0, pmids.len() as u32, None, true)
        .await
        .expect("esearch");
    let web_env = es.web_env.expect("no WebEnv");
    let query_key = es.query_key.expect("no QueryKey");
    let bulk_records = client
        .efetch_bulk(&web_env, query_key, 0, pmids.len() as u32)
        .await
        .expect("efetch_bulk");

    assert_eq!(
        bulk_records.len(),
        pmids.len(),
        "bulk returned {} records, expected {}",
        bulk_records.len(),
        pmids.len()
    );
    eprintln!(
        "[parity {}-pmid] indiv={} bulk={}",
        pmids.len(),
        indiv_records.len(),
        bulk_records.len()
    );

    // Compare every individual record with the matching bulk record.
    for indiv in &indiv_records {
        let from_bulk = bulk_records
            .iter()
            .find(|b| b.pmid == indiv.pmid)
            .unwrap_or_else(|| panic!("PMID {} missing from bulk response", indiv.pmid));
        assert_articles_equal(&format!("pmid={}", indiv.pmid), indiv, from_bulk);
    }
}
