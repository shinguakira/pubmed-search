//! Streaming PubMed XML walker.
//!
//! We don't deserialize the whole document — PubmedArticle records are
//! deeply nested and the fields we care about are scattered across many
//! element paths. So we walk events (`<x>`, `</x>`, text) with quick-xml
//! and keep a tiny stack (`path`) of the current ancestor element names.
//! When we see character data, we look at the top of the stack to decide
//! what to do with it (e.g. "we're inside `<ArticleTitle>`, so append").

use super::dto::{ArticleDetail, Author};

pub(super) fn parse_pubmed_xml(xml: &str, pmid: &str) -> anyhow::Result<ArticleDetail> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut title = String::new();
    let mut abstract_parts: Vec<(String, String)> = Vec::new();
    let mut current_abs_label = String::new();
    let mut authors: Vec<Author> = Vec::new();
    let mut journal = String::new();
    let mut pubdate_year = String::new();
    let mut pubdate_month = String::new();
    let mut pubdate_day = String::new();
    let mut doi = String::new();
    let mut keywords: Vec<String> = Vec::new();
    let mut mesh: Vec<String> = Vec::new();
    let mut pubtypes: Vec<String> = Vec::new();

    let mut path: Vec<String> = Vec::new();
    let mut cur_author: Option<Author> = None;
    let mut cur_id_type = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(anyhow::anyhow!("xml parse error: {e}")),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                path.push(name.clone());
                if name == "Author" {
                    cur_author = Some(Author {
                        last_name: String::new(),
                        fore_name: String::new(),
                        affiliation: String::new(),
                    });
                }
                if name == "AbstractText" {
                    current_abs_label.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Label" {
                            current_abs_label =
                                String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                if name == "ArticleId" {
                    cur_id_type.clear();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"IdType" {
                            cur_id_type = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "Author" {
                    if let Some(a) = cur_author.take() {
                        authors.push(a);
                    }
                }
                path.pop();
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().to_string();
                let in_path = |p: &str| path.iter().any(|x| x == p);
                let top = path.last().cloned().unwrap_or_default();
                match top.as_str() {
                    "ArticleTitle" => title.push_str(&text),
                    "AbstractText" => {
                        abstract_parts.push((current_abs_label.clone(), text));
                    }
                    "Title" if in_path("Journal") => journal = text,
                    "Year" if in_path("PubDate") => pubdate_year = text,
                    "Month" if in_path("PubDate") => pubdate_month = text,
                    "Day" if in_path("PubDate") => pubdate_day = text,
                    "LastName" => {
                        if let Some(a) = cur_author.as_mut() { a.last_name = text; }
                    }
                    "ForeName" => {
                        if let Some(a) = cur_author.as_mut() { a.fore_name = text; }
                    }
                    "Affiliation" => {
                        if let Some(a) = cur_author.as_mut() { a.affiliation = text; }
                    }
                    "Keyword" => keywords.push(text),
                    "DescriptorName" if in_path("MeshHeading") => mesh.push(text),
                    "PublicationType" => pubtypes.push(text),
                    "ArticleId" if cur_id_type == "doi" => doi = text,
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    let abstract_text = if abstract_parts.is_empty() {
        String::new()
    } else {
        abstract_parts
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

    let pubdate = [pubdate_year, pubdate_month, pubdate_day]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(ArticleDetail {
        pmid: pmid.to_string(),
        title,
        abstract_text,
        authors,
        journal,
        pubdate,
        doi,
        keywords,
        mesh_terms: mesh,
        pubtypes,
    })
}
