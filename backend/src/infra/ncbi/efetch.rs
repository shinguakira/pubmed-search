use super::client::{Client, EUTILS};
use super::dto::request::efetch::EfetchRequest;
use super::dto::response::ArticleDetail;
use super::xml::{parse_pubmed_xml, parse_pubmed_xml_bulk};

impl Client {
    /// Call NCBI `efetch.fcgi` for a single PMID and walk the returned
    /// PubmedArticle XML to pull out the fields we surface in the UI:
    /// title, structured abstract (BACKGROUND / METHODS / …), authors +
    /// affiliations, journal, pub date, DOI, keywords, MeSH terms,
    /// publication types.
    pub async fn efetch_abstract(&self, pmid: &str) -> anyhow::Result<ArticleDetail> {
        let req = EfetchRequest {
            db: "pubmed".into(),
            id: Some(pmid.into()),
            retmode: "xml",
            rettype: "abstract",
            web_env: None,
            query_key: None,
            retstart: None,
            retmax: None,
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/efetch.fcgi");
        let xml = self.http.get(url).query(&req).send().await?.text().await?;
        parse_pubmed_xml(&xml, pmid)
    }

    /// Bulk variant: fetch up to `retmax` records in a single HTTP call
    /// using the History server cursor returned by
    /// `esearch(..., use_history=true)`. NCBI allows up to 10,000
    /// records per request via this path. Records come back in NCBI's
    /// stored order.
    pub async fn efetch_bulk(
        &self,
        web_env: &str,
        query_key: u32,
        retstart: u32,
        retmax: u32,
    ) -> anyhow::Result<Vec<ArticleDetail>> {
        let req = EfetchRequest {
            db: "pubmed".into(),
            id: None,
            retmode: "xml",
            rettype: "abstract",
            web_env: Some(web_env.into()),
            query_key: Some(query_key),
            retstart: Some(retstart),
            retmax: Some(retmax),
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/efetch.fcgi");
        let xml = self.http.get(url).query(&req).send().await?.text().await?;
        parse_pubmed_xml_bulk(&xml)
    }
}
