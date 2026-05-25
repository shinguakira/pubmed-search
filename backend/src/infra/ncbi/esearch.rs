use super::client::{Client, EUTILS};
use super::dto::request::esearch::EsearchRequest;
use super::dto::response::EsearchResult;

/// NCBI sometimes returns JSON with raw control characters embedded
/// inside string values (their `ERROR` message is the canonical
/// offender). Replace any U+0000–U+001F char with a space so
/// `serde_json` stops rejecting the payload.
fn sanitize_json_controls(s: &str) -> String {
    s.chars()
        .map(|c| if (c as u32) < 0x20 && c != '\n' && c != '\r' && c != '\t' {
            ' '
        } else if matches!(c, '\n' | '\r' | '\t') {
            ' '
        } else { c })
        .collect()
}

impl Client {
    /// Call NCBI `esearch.fcgi`. Returns the IDs that match `term`, the
    /// total result count, the human-readable translation NCBI used
    /// (e.g. expanding `crispr` to its MeSH synonyms), and — when
    /// `use_history` is true — a `WebEnv` + `QueryKey` cursor that
    /// `efetch_bulk` can consume to pull large batches in one HTTP call.
    pub async fn esearch(
        &self,
        db: &str,
        term: &str,
        retstart: u32,
        retmax: u32,
        sort: Option<&str>,
        use_history: bool,
    ) -> anyhow::Result<EsearchResult> {
        let req = EsearchRequest {
            db: db.into(),
            term: term.into(),
            retstart,
            retmax,
            retmode: "json",
            sort: sort.map(String::from),
            usehistory: if use_history { Some("y") } else { None },
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/esearch.fcgi");
        // NCBI's esearchresult can contain an "ERROR" field with an
        // unescaped newline in the message (e.g. "Exception:\n'retstart'
        // cannot be larger than 9998 …"), which makes serde_json reject
        // the body. Pull the raw text first, then strip raw control
        // chars inside string values before parsing.
        let raw = self.http.post(url).form(&req).send().await?.text().await?;
        let cleaned = sanitize_json_controls(&raw);
        let body: serde_json::Value = serde_json::from_str(&cleaned)?;

        let result = &body["esearchresult"];
        if let Some(err) = result["ERROR"].as_str() {
            return Err(anyhow::anyhow!("NCBI esearch error: {err}"));
        }
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
        let web_env = result["webenv"].as_str().map(String::from);
        let query_key = result["querykey"]
            .as_str()
            .and_then(|s| s.parse().ok());

        Ok(EsearchResult {
            count,
            ids,
            querytranslation,
            web_env,
            query_key,
        })
    }
}
