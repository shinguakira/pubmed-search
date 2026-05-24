use std::collections::HashMap;

use super::client::{Client, EUTILS};
use super::dto::request::esummary::EsummaryRequest;
use super::dto::response::Summary;

impl Client {
    /// Call NCBI `esummary.fcgi`. Hydrates a batch of PMIDs into the
    /// short metadata shown in the results list (title/authors/journal/…).
    pub async fn esummary(&self, db: &str, ids: &[String]) -> anyhow::Result<Vec<Summary>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let req = EsummaryRequest {
            db: db.into(),
            id: ids.join(","),
            retmode: "json",
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/esummary.fcgi");
        let body: serde_json::Value =
            self.http.get(url).query(&req).send().await?.json().await?;

        let result = &body["result"];
        let uids: Vec<String> = result["uids"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
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
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
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
}
