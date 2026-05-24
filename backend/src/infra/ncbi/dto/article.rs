use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Full article record parsed out of NCBI `efetch` XML.
///
/// Returned by `GET /api/article/{pmid}`. Compared to `Summary`,
/// this carries the abstract text, author affiliations, MeSH terms,
/// and author-supplied keywords.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArticleDetail {
    /// PubMed unique identifier, echoed back from the request.
    pub pmid: String,

    /// Article title. May contain inline HTML tags from the source record.
    pub title: String,

    /// Full abstract. Multi-section abstracts (`BACKGROUND`, `METHODS`,
    /// `RESULTS`, `CONCLUSIONS`, …) are joined with a blank line, each
    /// section prefixed with its label. Empty string if no abstract is
    /// indexed for this PMID.
    pub abstract_text: String,

    /// Authors with full names + affiliation strings, in published order.
    pub authors: Vec<Author>,

    /// Full journal title (e.g. `Cell communication and signaling : CCS`).
    pub journal: String,

    /// Publication date as `YYYY [Month [Day]]`, e.g. `2026 May 23`.
    /// Components are omitted if NCBI didn't index them.
    pub pubdate: String,

    /// DOI without the `https://doi.org/` prefix. Empty if missing.
    pub doi: String,

    /// Author-supplied keywords (NCBI calls these "Keyword List";
    /// distinct from MeSH terms).
    pub keywords: Vec<String>,

    /// NLM-assigned MeSH descriptors (controlled vocabulary). Empty if
    /// the record hasn't been indexed yet (recent articles often haven't).
    pub mesh_terms: Vec<String>,

    /// Publication-type labels (`Journal Article`, `Review`, …). Same
    /// shape as `Summary.pubtypes`.
    pub pubtypes: Vec<String>,
}

/// One author record from `efetch` XML.
///
/// All fields default to empty string when NCBI omits them — typically
/// when an author has only a collective name, or when affiliation isn't
/// indexed.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Author {
    /// Family name as it appears in the article.
    pub last_name: String,

    /// Given names + middle initials, space-separated (e.g. `John A`).
    pub fore_name: String,

    /// Free-text affiliation string. May contain a full address, ORCID,
    /// or email — we preserve NCBI's content verbatim.
    pub affiliation: String,
}
