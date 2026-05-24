use super::client::{Client, EUTILS};
use super::dto::request::efetch::EfetchRequest;
use super::dto::response::ArticleDetail;
use super::xml::parse_pubmed_xml;

impl Client {
    /// Call NCBI `efetch.fcgi` for a single PMID and walk the returned
    /// PubmedArticle XML to pull out the fields we surface in the UI:
    /// title, structured abstract (BACKGROUND / METHODS / …), authors +
    /// affiliations, journal, pub date, DOI, keywords, MeSH terms,
    /// publication types.
    pub async fn efetch_abstract(&self, pmid: &str) -> anyhow::Result<ArticleDetail> {
        let req = EfetchRequest {
            db: "pubmed".into(),
            id: pmid.into(),
            retmode: "xml",
            rettype: "abstract",
            ident: self.ident(),
        };
        let url = format!("{EUTILS}/efetch.fcgi");
        let xml = self.http.get(url).query(&req).send().await?.text().await?;
        parse_pubmed_xml(&xml, pmid)
    }
}
