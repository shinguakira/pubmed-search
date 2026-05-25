use serde::Serialize;
use utoipa::ToSchema;

/// One row in the search results — the data NCBI `esummary` returns for
/// a single PMID, normalized.
///
/// Most string fields default to `""` when NCBI omits them rather than
/// being `Option<String>`, so the frontend can render unconditionally
/// without null checks.
#[derive(Debug, Serialize, ToSchema)]
pub struct Summary {
    /// PubMed unique identifier, e.g. `25315507`.
    pub pmid: String,

    /// Article title. May contain inline HTML tags (`<i>`, `<sub>`, …)
    /// that NCBI preserves from the original record.
    pub title: String,

    /// Short author names in NCBI's display form
    /// (`Last F`, e.g. `Liu Y`). Use `efetch_abstract` for full names.
    pub authors: Vec<String>,

    /// Journal abbreviation (e.g. `Nat Med`, `Cell`). NCBI's "source"
    /// field.
    pub source: String,

    /// Publication date as NCBI formats it (e.g. `2026 May 23`,
    /// `2024 Jun`). Free-text — *not* ISO-8601.
    pub pubdate: String,

    /// Electronic-ahead-of-print date if applicable, otherwise empty.
    /// Same free-text format as `pubdate`.
    pub epubdate: String,

    /// Journal volume number.
    pub volume: String,

    /// Journal issue number within the volume.
    pub issue: String,

    /// Page range within the issue (e.g. `5186-93`).
    pub pages: String,

    /// DOI without the `https://doi.org/` prefix
    /// (e.g. `10.1038/s41586-024-12345`). Empty if NCBI didn't index one.
    pub doi: String,

    /// Publication-type labels from NCBI
    /// (e.g. `Journal Article`, `Review`, `Randomized Controlled Trial`).
    pub pubtypes: Vec<String>,

    /// ISO 639-2 language code (`eng`, `jpn`, …). Empty if missing.
    pub lang: String,

    /// Abstract text. Populated only when the search request used
    /// `bulk=true` (the bulk path fetches the full record via efetch).
    /// `None` on the default esummary path, which doesn't include
    /// abstracts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
}
