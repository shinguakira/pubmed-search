//! Fixture-value tests — assert *specific* field values for known
//! PMIDs in **both** the individual and bulk fetch paths. Catches
//! parser regressions that the field-by-field parity test alone can't
//! (both paths could agree on a wrongly-parsed value).
//!
//! Fixtures cover two stable records:
//!   * 13054692 — Watson & Crick 1953, no DOI, no abstract, MeSH later
//!   * 25315507 — Ma Y et al. 2014 (CRISPR/Cas9), modern record with
//!                DOI, structured abstract, full MeSH

use pubmed_backend::infra::ncbi::{ArticleDetail, Client};
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

struct Fixture {
    pmid: &'static str,
    /// Substring expected somewhere in title (case-insensitive).
    title_contains: &'static str,
    /// Exact journal abbreviation NCBI returns.
    journal: &'static str,
    /// Year prefix the pubdate string should start with.
    pubdate_year: &'static str,
    author_count: usize,
    /// Lowercased expected last name of the first author.
    first_author_last_lower: &'static str,
    /// Some(exact) or None when NCBI has no DOI for the record.
    doi: Option<&'static str>,
    has_abstract: bool,
    has_mesh: bool,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        pmid: "13054692",
        title_contains: "nucleic acid",
        journal: "Nature",
        pubdate_year: "1953",
        author_count: 2,
        first_author_last_lower: "watson",
        // Nature assigned DOIs retroactively to historical papers.
        doi: Some("10.1038/171737a0"),
        has_abstract: false,
        has_mesh: true,
    },
    Fixture {
        pmid: "25315507",
        title_contains: "CRISPR",
        // `efetch` returns the full journal title; `esummary` returns
        // the abbreviation ("FEBS J"). Our XML parser reads efetch, so
        // we expect the full form here.
        journal: "The FEBS journal",
        pubdate_year: "2014",
        author_count: 3,
        first_author_last_lower: "ma",
        doi: Some("10.1111/febs.13110"),
        has_abstract: true,
        has_mesh: true,
    },
];

async fn fetch_indiv(client: &Client, pmid: &str) -> ArticleDetail {
    client.efetch_abstract(pmid).await.expect("efetch_abstract failed")
}

async fn fetch_bulk(client: &Client, pmid: &str) -> ArticleDetail {
    let term = format!("{pmid}[uid]");
    let es = client
        .esearch("pubmed", &term, 0, 1, None, true)
        .await
        .expect("esearch failed");
    let web_env = es.web_env.expect("no WebEnv");
    let query_key = es.query_key.expect("no QueryKey");
    let mut records = client
        .efetch_bulk(&web_env, query_key, 0, 1)
        .await
        .expect("efetch_bulk failed");
    assert_eq!(records.len(), 1, "bulk did not return exactly 1 record for {pmid}");
    records.remove(0)
}

fn assert_fixture(path: &str, fx: &Fixture, art: &ArticleDetail) {
    assert_eq!(art.pmid, fx.pmid, "[{path}/{}] pmid", fx.pmid);

    assert!(
        art.title
            .to_lowercase()
            .contains(&fx.title_contains.to_lowercase()),
        "[{path}/{}] title `{}` should contain `{}`",
        fx.pmid, art.title, fx.title_contains
    );

    assert_eq!(
        art.journal, fx.journal,
        "[{path}/{}] journal `{}` != `{}`",
        fx.pmid, art.journal, fx.journal
    );

    assert!(
        art.pubdate.starts_with(fx.pubdate_year),
        "[{path}/{}] pubdate `{}` should start with `{}`",
        fx.pmid, art.pubdate, fx.pubdate_year
    );

    assert_eq!(
        art.authors.len(),
        fx.author_count,
        "[{path}/{}] author count {} != {}",
        fx.pmid,
        art.authors.len(),
        fx.author_count
    );

    assert_eq!(
        art.authors[0].last_name.to_lowercase(),
        fx.first_author_last_lower,
        "[{path}/{}] first author last_name `{}` != `{}`",
        fx.pmid, art.authors[0].last_name, fx.first_author_last_lower
    );

    match fx.doi {
        Some(expected) => assert_eq!(
            art.doi, expected,
            "[{path}/{}] doi `{}` != `{}`",
            fx.pmid, art.doi, expected
        ),
        None => assert!(
            art.doi.is_empty(),
            "[{path}/{}] doi should be empty, got `{}`",
            fx.pmid, art.doi
        ),
    }

    assert_eq!(
        !art.abstract_text.is_empty(),
        fx.has_abstract,
        "[{path}/{}] has_abstract={} expected={} (len={})",
        fx.pmid,
        !art.abstract_text.is_empty(),
        fx.has_abstract,
        art.abstract_text.len()
    );

    assert_eq!(
        !art.mesh_terms.is_empty(),
        fx.has_mesh,
        "[{path}/{}] has_mesh={} expected={} (n={})",
        fx.pmid,
        !art.mesh_terms.is_empty(),
        fx.has_mesh,
        art.mesh_terms.len()
    );
}

#[tokio::test]
async fn fixtures_match_indiv_and_bulk() {
    let _gate = ncbi_gate().lock().await;
    if !ncbi_reachable().await {
        eprintln!("[skip] NCBI not reachable");
        return;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;

    let client = Client::new();

    for fx in FIXTURES {
        let indiv = fetch_indiv(&client, fx.pmid).await;
        tokio::time::sleep(Duration::from_millis(400)).await;
        let bulk = fetch_bulk(&client, fx.pmid).await;
        tokio::time::sleep(Duration::from_millis(400)).await;

        eprintln!(
            "[fixture] pmid={} title=`{}` journal={} pubdate=`{}` \
             authors={} doi={} abs_len={} mesh={}",
            fx.pmid,
            indiv.title,
            indiv.journal,
            indiv.pubdate,
            indiv.authors.len(),
            if indiv.doi.is_empty() { "—" } else { &indiv.doi },
            indiv.abstract_text.len(),
            indiv.mesh_terms.len()
        );

        assert_fixture("indiv", fx, &indiv);
        assert_fixture("bulk", fx, &bulk);
    }
}
