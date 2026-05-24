use super::client::{Client, EUTILS};
use super::dto::request::esearch::EsearchRequest;
use super::dto::response::EsearchResult;

impl Client {
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
        let req = EsearchRequest {
            db: db.into(),
            term: term.into(),
            retstart,
            retmax,
            retmode: "json",
            sort: sort.map(String::from),
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/esearch.fcgi");
        let body: serde_json::Value =
            self.http.get(url).query(&req).send().await?.json().await?;

        let result = &body["esearchresult"];
        let count: u32 = result["count"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let ids: Vec<String> = result["idlist"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let querytranslation = result["querytranslation"].as_str().unwrap_or("").to_string();
        Ok(EsearchResult {
            count,
            ids,
            querytranslation,
        })
    }
}
