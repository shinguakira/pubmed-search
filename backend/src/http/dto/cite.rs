use serde::Serialize;
use utoipa::ToSchema;

/// Response body for `GET /api/cite/{pmid}` — the same citation
/// rendered five different ways.
///
/// All formats are generated server-side from one `efetch` call and are
/// plain text (no HTML). Wrap in `<pre>` on the client if you want to
/// preserve line breaks (BibTeX is multi-line).
#[derive(Debug, Serialize, ToSchema)]
pub struct CiteResponse {
    /// AMA 11th-edition style. One-line.
    pub ama: String,

    /// APA 7th-edition style. One-line.
    pub apa: String,

    /// MLA 9th-edition style. One-line.
    pub mla: String,

    /// NLM journal-citation style (similar to AMA; used by PubMed
    /// itself).
    pub nlm: String,

    /// BibTeX entry, ready to paste into a `.bib` file. Multi-line.
    /// The cite key is `pmid{PMID}`.
    pub bibtex: String,
}
