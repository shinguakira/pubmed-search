use super::client::{Client, EUTILS};
use super::types::ArticleDetail;
use super::xml::parse_pubmed_xml;

impl Client {
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
