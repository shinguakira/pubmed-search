//! NCBI E-utilities client + a small XML parser for `efetch` responses.
//!
//! This module is the only place in the crate that talks to the public
//! internet. Everything else (routes, OpenAPI, tests) goes through `Client`.
//!
//! What each upstream endpoint does (NCBI naming):
//!
//! * **esearch** — given a query string, returns a list of PMIDs + total hit count.
//! * **esummary** — given PMIDs, returns short metadata (title, authors, journal).
//! * **efetch**  — given a single PMID, returns the full record as XML
//!   (abstract, MeSH, affiliations…). We parse only the fields we need.
//!
//! `Client` is `Clone` so Axum can hand a copy to each request via `State`
//! without locking — `reqwest::Client` itself is cheap to clone (it shares
//! the underlying connection pool internally).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

const EUTILS: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: Option<String>,
    tool: String,
    email: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("pubmed-search-poc/0.1")
                .build()
                .expect("reqwest client"),
            api_key: std::env::var("NCBI_API_KEY").ok(),
            tool: std::env::var("NCBI_TOOL").unwrap_or_else(|_| "pubmed-search-poc".into()),
            email: std::env::var("NCBI_EMAIL").unwrap_or_else(|_| "dev@example.com".into()),
        }
    }

    fn base_params(&self) -> Vec<(&'static str, String)> {
        let mut v = vec![
            ("tool", self.tool.clone()),
            ("email", self.email.clone()),
        ];
        if let Some(k) = &self.api_key {
            v.push(("api_key", k.clone()));
        }
        v
    }

    /// Call NCBI `esearch.fcgi`. Returns the IDs that match `term`, the
    /// total result count, and the human-readable translation NCBI used
    /// (e.g. expanding `crispr` to its MeSH synonyms).
    pub async fn esearch(
        &self,
        db: &str,
        term: &str,
        retstart: u32,
        retmax: u32,
        sort: Option<&str>,
    ) -> anyhow::Result<EsearchResult> {
        let mut params = self.base_params();
        params.push(("db", db.into()));
        params.push(("term", term.into()));
        params.push(("retmode", "json".into()));
        params.push(("retstart", retstart.to_string()));
        params.push(("retmax", retmax.to_string()));
        if let Some(s) = sort {
            params.push(("sort", s.into()));
        }
        let url = format!("{EUTILS}/esearch.fcgi");
        let body: serde_json::Value = self.http.get(url).query(&params).send().await?.json().await?;
        let result = &body["esearchresult"];
        let count: u32 = result["count"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
        let ids: Vec<String> = result["idlist"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let querytranslation = result["querytranslation"].as_str().unwrap_or("").to_string();
        Ok(EsearchResult { count, ids, querytranslation })
    }

    /// Call NCBI `esummary.fcgi`. Hydrates a batch of PMIDs into the
    /// short metadata shown in the results list (title/authors/journal/…).
    pub async fn esummary(&self, db: &str, ids: &[String]) -> anyhow::Result<Vec<Summary>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut params = self.base_params();
        params.push(("db", db.into()));
        params.push(("id", ids.join(",")));
        params.push(("retmode", "json".into()));
        let url = format!("{EUTILS}/esummary.fcgi");
        let body: serde_json::Value = self.http.get(url).query(&params).send().await?.json().await?;
        let result = &body["result"];
        let uids: Vec<String> = result["uids"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let mut out = Vec::with_capacity(uids.len());
        for uid in uids {
            let s = &result[&uid];
            let authors: Vec<String> = s["authors"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|au| au["name"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let article_ids: HashMap<String, String> = s["articleids"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| {
                            let t = x["idtype"].as_str()?.to_string();
                            let v = x["value"].as_str()?.to_string();
                            Some((t, v))
                        })
                        .collect()
                })
                .unwrap_or_default();
            out.push(Summary {
                pmid: uid,
                title: s["title"].as_str().unwrap_or("").to_string(),
                authors,
                source: s["source"].as_str().unwrap_or("").to_string(),
                pubdate: s["pubdate"].as_str().unwrap_or("").to_string(),
                epubdate: s["epubdate"].as_str().unwrap_or("").to_string(),
                volume: s["volume"].as_str().unwrap_or("").to_string(),
                issue: s["issue"].as_str().unwrap_or("").to_string(),
                pages: s["pages"].as_str().unwrap_or("").to_string(),
                doi: article_ids.get("doi").cloned().unwrap_or_default(),
                pubtypes: s["pubtype"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                lang: s["lang"]
                    .as_array()
                    .and_then(|a| a.first().and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string(),
            });
        }
        Ok(out)
    }

    /// Call NCBI `efetch.fcgi` for a single PMID and walk the returned
    /// PubmedArticle XML to pull out the fields we surface in the UI:
    /// title, structured abstract (BACKGROUND / METHODS / …), authors +
    /// affiliations, journal, pub date, DOI, keywords, MeSH terms,
    /// publication types.
    pub async fn efetch_abstract(&self, pmid: &str) -> anyhow::Result<ArticleDetail> {
        let mut params = self.base_params();
        params.push(("db", "pubmed".into()));
        params.push(("id", pmid.into()));
        params.push(("retmode", "xml".into()));
        params.push(("rettype", "abstract".into()));
        let url = format!("{EUTILS}/efetch.fcgi");
        let xml = self.http.get(url).query(&params).send().await?.text().await?;
        parse_pubmed_xml(&xml, pmid)
    }
}

#[derive(Debug, Serialize)]
pub struct EsearchResult {
    pub count: u32,
    pub ids: Vec<String>,
    pub querytranslation: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Summary {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub source: String,
    pub pubdate: String,
    pub epubdate: String,
    pub volume: String,
    pub issue: String,
    pub pages: String,
    pub doi: String,
    pub pubtypes: Vec<String>,
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArticleDetail {
    pub pmid: String,
    pub title: String,
    pub abstract_text: String,
    pub authors: Vec<Author>,
    pub journal: String,
    pub pubdate: String,
    pub doi: String,
    pub keywords: Vec<String>,
    pub mesh_terms: Vec<String>,
    pub pubtypes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Author {
    pub last_name: String,
    pub fore_name: String,
    pub affiliation: String,
}

/// Streaming PubMed XML walker.
///
/// We don't deserialize the whole document — PubmedArticle records are
/// deeply nested and the fields we care about are scattered across many
/// element paths. So we walk events (`<x>`, `</x>`, text) with quick-xml
/// and keep a tiny stack (`path`) of the current ancestor element names.
/// When we see character data, we look at the top of the stack to decide
/// what to do with it (e.g. "we're inside `<ArticleTitle>`, so append").
fn parse_pubmed_xml(xml: &str, pmid: &str) -> anyhow::Result<ArticleDetail> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut title = String::new();
    let mut abstract_parts: Vec<(String, String)> = Vec::new();
    let mut current_abs_label = String::new();
    let mut authors: Vec<Author> = Vec::new();
    let mut journal = String::new();
    let mut pubdate_year = String::new();
    let mut pubdate_month = String::new();
    let mut pubdate_day = String::new();
    let mut doi = String::new();
    let mut keywords: Vec<String> = Vec::new();
    let mut mesh: Vec<String> = Vec::new();
    let mut pubtypes: Vec<String> = Vec::new();

    let mut path: Vec<String> = Vec::new();
    let mut cur_author: Option<Author> = None;
    let mut cur_id_type = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow::anyhow!("xml parse error: {e}")),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                path.push(name.clone());
                if name == "Author" {
                    cur_author = Some(Author {
                        last_name: String::new(),
                        fore_name: String::new(),
                        affiliation: String::new(),
                    });
                }
                if name == "AbstractText" {
                    current_abs_label.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Label" {
                            current_abs_label =
                                String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                if name == "ArticleId" {
                    cur_id_type.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"IdType" {
                            cur_id_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "Author" {
                    if let Some(a) = cur_author.take() {
                        authors.push(a);
                    }
                }
                path.pop();
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().to_string();
                let in_path = |p: &str| path.iter().any(|x| x == p);
                let top = path.last().cloned().unwrap_or_default();
                match top.as_str() {
                    "ArticleTitle" => title.push_str(&text),
                    "AbstractText" => {
                        abstract_parts.push((current_abs_label.clone(), text));
                    }
                    "Title" if in_path("Journal") => journal = text,
                    "Year" if in_path("PubDate") => pubdate_year = text,
                    "Month" if in_path("PubDate") => pubdate_month = text,
                    "Day" if in_path("PubDate") => pubdate_day = text,
                    "LastName" => {
                        if let Some(a) = cur_author.as_mut() { a.last_name = text; }
                    }
                    "ForeName" => {
                        if let Some(a) = cur_author.as_mut() { a.fore_name = text; }
                    }
                    "Affiliation" => {
                        if let Some(a) = cur_author.as_mut() { a.affiliation = text; }
                    }
                    "Keyword" => keywords.push(text),
                    "DescriptorName" if in_path("MeshHeading") => mesh.push(text),
                    "PublicationType" => pubtypes.push(text),
                    "ArticleId" if cur_id_type == "doi" => doi = text,
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    let abstract_text = if abstract_parts.is_empty() {
        String::new()
    } else {
        abstract_parts
            .into_iter()
            .map(|(label, text)| {
                if label.is_empty() {
                    text
                } else {
                    format!("{}: {}", label, text)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    let pubdate = [pubdate_year, pubdate_month, pubdate_day]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(ArticleDetail {
        pmid: pmid.to_string(),
        title,
        abstract_text,
        authors,
        journal,
        pubdate,
        doi,
        keywords,
        mesh_terms: mesh,
        pubtypes,
    })
}
