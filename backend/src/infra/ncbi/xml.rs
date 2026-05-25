//! Streaming PubMed XML walker.
//!
//! `efetch.fcgi` returns one or more `<PubmedArticle>` records wrapped
//! in `<PubmedArticleSet>`. We walk events (`<x>`, `</x>`, text) with
//! quick-xml and keep a tiny stack (`path`) of the current ancestor
//! element names. When we see character data, we look at the top of the
//! stack to decide what to do with it.
//!
//! Two entry points:
//! * [`parse_pubmed_xml_bulk`] — returns every `<PubmedArticle>` in the
//!   document. Used by `Client::efetch_bulk`.
//! * [`parse_pubmed_xml`] — convenience for the single-PMID case; runs
//!   the bulk parser and picks the matching record (falls back to the
//!   first one).

use super::dto::response::{ArticleDetail, Author, Reference};

pub(super) fn parse_pubmed_xml(xml: &str, pmid: &str) -> anyhow::Result<ArticleDetail> {
    let mut all = parse_pubmed_xml_bulk(xml)?;
    if all.is_empty() {
        return Err(anyhow::anyhow!("no PubmedArticle in efetch response"));
    }
    let idx = all.iter().position(|a| a.pmid == pmid).unwrap_or(0);
    Ok(all.swap_remove(idx))
}

pub(super) fn parse_pubmed_xml_bulk(xml: &str) -> anyhow::Result<Vec<ArticleDetail>> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut out: Vec<ArticleDetail> = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut cur: Option<ArticleBuilder> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow::anyhow!("xml parse error: {e}")),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                path.push(name.clone());

                if name == "PubmedArticle" {
                    cur = Some(ArticleBuilder::default());
                    continue;
                }

                let Some(b) = cur.as_mut() else { continue };

                if name == "Author" {
                    b.cur_author = Some(Author {
                        last_name: String::new(),
                        fore_name: String::new(),
                        affiliation: String::new(),
                    });
                }
                if name == "Reference" {
                    b.cur_reference = Some(Reference {
                        citation: String::new(),
                        pmid: None,
                        doi: None,
                    });
                }
                if name == "AbstractText" {
                    b.current_abs_label.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Label" {
                            b.current_abs_label =
                                String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                if name == "ArticleId" {
                    b.cur_id_type.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"IdType" {
                            b.cur_id_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "PubmedArticle" {
                    if let Some(b) = cur.take() {
                        out.push(b.build());
                    }
                } else if name == "Author" {
                    if let Some(b) = cur.as_mut() {
                        if let Some(a) = b.cur_author.take() {
                            b.authors.push(a);
                        }
                    }
                } else if name == "Reference" {
                    if let Some(b) = cur.as_mut() {
                        if let Some(r) = b.cur_reference.take() {
                            b.references.push(r);
                        }
                    }
                }
                path.pop();
            }
            Ok(Event::Text(t)) => {
                let Some(b) = cur.as_mut() else { continue };
                let text = t.unescape().unwrap_or_default().to_string();
                let in_path = |p: &str| path.iter().any(|x| x == p);
                // Parent of the current text node (top of stack is the
                // element itself; element-1 is its container).
                let parent = path
                    .get(path.len().saturating_sub(2))
                    .cloned()
                    .unwrap_or_default();
                let top = path.last().cloned().unwrap_or_default();

                // References live inside <ReferenceList><Reference>… and
                // are handled before the generic article-level matchers
                // so their nested elements don't bleed into article
                // fields (the cited paper's title is not the article's
                // title, etc.).
                if let Some(r) = b.cur_reference.as_mut() {
                    match top.as_str() {
                        "Citation" => r.citation.push_str(&text),
                        "ArticleId" if b.cur_id_type == "pubmed" => {
                            r.pmid = Some(text);
                        }
                        "ArticleId" if b.cur_id_type == "doi" => {
                            r.doi = Some(text);
                        }
                        _ => {}
                    }
                    continue;
                }

                match top.as_str() {
                    "PMID" if parent == "MedlineCitation" => b.pmid = text,
                    "ArticleTitle" => b.title.push_str(&text),
                    "AbstractText" => {
                        b.abstract_parts.push((b.current_abs_label.clone(), text));
                    }
                    "Title" if in_path("Journal") => b.journal = text,
                    "Year" if in_path("PubDate") => b.pubdate_year = text,
                    "Month" if in_path("PubDate") => b.pubdate_month = text,
                    "Day" if in_path("PubDate") => b.pubdate_day = text,
                    "LastName" => {
                        if let Some(a) = b.cur_author.as_mut() {
                            a.last_name = text;
                        }
                    }
                    "ForeName" => {
                        if let Some(a) = b.cur_author.as_mut() {
                            a.fore_name = text;
                        }
                    }
                    "Affiliation" => {
                        if let Some(a) = b.cur_author.as_mut() {
                            a.affiliation = text;
                        }
                    }
                    "Keyword" => b.keywords.push(text),
                    "DescriptorName" if in_path("MeshHeading") => b.mesh.push(text),
                    "PublicationType" => b.pubtypes.push(text),
                    "ArticleId" if b.cur_id_type == "doi" => b.doi = text,
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(out)
}

#[derive(Default)]
struct ArticleBuilder {
    pmid: String,
    title: String,
    abstract_parts: Vec<(String, String)>,
    current_abs_label: String,
    authors: Vec<Author>,
    cur_author: Option<Author>,
    cur_id_type: String,
    journal: String,
    pubdate_year: String,
    pubdate_month: String,
    pubdate_day: String,
    doi: String,
    keywords: Vec<String>,
    mesh: Vec<String>,
    pubtypes: Vec<String>,
    references: Vec<Reference>,
    cur_reference: Option<Reference>,
}

impl ArticleBuilder {
    fn build(self) -> ArticleDetail {
        let abstract_text = if self.abstract_parts.is_empty() {
            String::new()
        } else {
            self.abstract_parts
                .into_iter()
                .map(|(label, text)| {
                    if label.is_empty() {
                        text
                    } else {
                        format!("{}: {}", label, text)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let pubdate = [self.pubdate_year, self.pubdate_month, self.pubdate_day]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        ArticleDetail {
            pmid: self.pmid,
            title: self.title,
            abstract_text,
            authors: self.authors,
            journal: self.journal,
            pubdate,
            doi: self.doi,
            keywords: self.keywords,
            mesh_terms: self.mesh,
            pubtypes: self.pubtypes,
            references: self.references,
        }
    }
}
