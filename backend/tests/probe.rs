//! Manual probe: hit NCBI esearch via our reqwest setup and dump the
//! raw response bytes. Just for debugging the bench failure.

use std::time::Duration;

#[tokio::test]
#[ignore = "manual debug"]
async fn probe_esearch_raw() {
    let http = reqwest::Client::builder()
        .user_agent("pubmed-search-poc/0.1")
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let params = [
        ("db", "pubmed"),
        ("term", "crispr cas9"),
        ("retstart", "10000"),
        ("retmax", "10000"),
        ("retmode", "json"),
        ("tool", "pubmed-search-poc"),
        ("email", "dev@example.com"),
    ];

    let res = http
        .post("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi")
        .form(&params)
        .send()
        .await
        .unwrap();

    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.bytes().await.unwrap();

    eprintln!("status: {status}");
    eprintln!("headers:");
    for (k, v) in headers.iter() {
        eprintln!("  {}: {}", k, v.to_str().unwrap_or("(non-utf8)"));
    }
    eprintln!("body length: {}", bytes.len());
    eprintln!("first 200 bytes (hex):");
    for chunk in bytes.iter().take(200).enumerate() {
        eprint!("{:02x} ", chunk.1);
        if (chunk.0 + 1) % 16 == 0 {
            eprintln!();
        }
    }
    eprintln!();
    eprintln!("first 200 bytes (lossy str): {}", String::from_utf8_lossy(&bytes[..bytes.len().min(200)]));
}
