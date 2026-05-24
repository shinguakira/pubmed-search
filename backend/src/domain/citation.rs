//! Render a list of `ArticleDetail` records into a downloadable
//! format. These are pure functions — no IO — so the export handler
//! just calls them and wraps the result in an HTTP response.

use crate::infra::ncbi::ArticleDetail;

/// Render every article as a BibTeX `@article` entry, separated by a
/// blank line. The cite key is `pmid{PMID}`.
pub fn to_bibtex(articles: &[ArticleDetail]) -> String {
    let mut out = String::new();
    for d in articles {
        let year = d.pubdate.split_whitespace().next().unwrap_or("");
        let authors = d
            .authors
            .iter()
            .map(|a| format!("{}, {}", a.last_name, a.fore_name))
            .collect::<Vec<_>>()
            .join(" and ");
        out.push_str(&format!(
            "@article{{pmid{pmid},\n  title   = {{ {title} }},\n  author  = {{ {authors} }},\n  journal = {{ {journal} }},\n  year    = {{ {year} }},\n  doi     = {{ {doi} }},\n  pmid    = {{ {pmid} }}\n}}\n\n",
            pmid = d.pmid,
            title = d.title,
            authors = authors,
            journal = d.journal,
            year = year,
            doi = d.doi
        ));
    }
    out
}

/// Render as RFC 4180 CSV with a header row. Authors are joined with
/// `; ` inside one cell. Fields containing commas, quotes, or newlines
/// are wrapped in double quotes with inner quotes doubled.
pub fn to_csv(articles: &[ArticleDetail]) -> String {
    let mut out = String::from("PMID,Title,Authors,Journal,PubDate,DOI\n");
    for d in articles {
        let authors = d
            .authors
            .iter()
            .map(|a| format!("{} {}", a.last_name, a.fore_name).trim().to_string())
            .collect::<Vec<_>>()
            .join("; ");
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            csv_escape(&d.pmid),
            csv_escape(&d.title),
            csv_escape(&authors),
            csv_escape(&d.journal),
            csv_escape(&d.pubdate),
            csv_escape(&d.doi),
        ));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::ncbi::{ArticleDetail, Author};

    fn sample() -> ArticleDetail {
        ArticleDetail {
            pmid: "12345".into(),
            title: "A title, with comma".into(),
            abstract_text: "...".into(),
            authors: vec![
                Author {
                    last_name: "Smith".into(),
                    fore_name: "Jane A".into(),
                    affiliation: "".into(),
                },
                Author {
                    last_name: "Doe".into(),
                    fore_name: "John".into(),
                    affiliation: "".into(),
                },
            ],
            journal: "Nat Med".into(),
            pubdate: "2025 May 1".into(),
            doi: "10.1/test".into(),
            keywords: vec![],
            mesh_terms: vec![],
            pubtypes: vec!["Journal Article".into()],
        }
    }

    #[test]
    fn bibtex_emits_one_entry_per_article() {
        let out = to_bibtex(&[sample(), sample()]);
        assert_eq!(out.matches("@article{pmid").count(), 2);
        assert!(out.contains("pmid12345"));
        assert!(out.contains("Smith, Jane A and Doe, John"));
    }

    #[test]
    fn csv_escapes_commas_and_has_header() {
        let out = to_csv(&[sample()]);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "PMID,Title,Authors,Journal,PubDate,DOI");
        // Title contains a comma so it should be quoted.
        assert!(lines[1].contains("\"A title, with comma\""));
        // Authors cell contains the "; " separator.
        assert!(lines[1].contains("Smith Jane A; Doe John"));
    }

    #[test]
    fn csv_escapes_inner_quotes() {
        let mut a = sample();
        a.title = r#"Quoted "thing""#.into();
        let out = to_csv(&[a]);
        assert!(out.contains("\"Quoted \"\"thing\"\"\""));
    }
}
