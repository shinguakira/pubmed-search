# Data flow: default search vs bulk search

Two end-to-end traces of **what happens when the user runs a search**.
Same UI, same DTOs — the only difference is the `FETCH MODE` toggle
(`Default` / `Bulk`) under the search input.

Layer prefixes used throughout:
* **FE** — React frontend (`frontend/src/…`)
* **BE** — Rust backend (`backend/src/…`)
* **NCBI** — public NCBI E-utilities

---

## 1. Default search (`bulk=false`)

Two upstream NCBI hops. The cheap, lightweight path. Result rows do
**not** carry abstracts.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant FE as FE / App.tsx
    participant API as FE / lib/api.ts
    participant BE as BE / http/search.rs
    participant CLI as BE / infra/ncbi
    participant NCBI as NCBI

    User->>FE: type "crispr" → click Search
    FE->>FE: applySearch() — write URL ?q=crispr&page=1
    FE->>FE: useEffect on URL change
    FE->>API: search({term, page, pageSize, sort, filters, bulk:false})
    API->>BE: GET /api/search?term=crispr&page=1&page_size=20
    BE->>BE: combine term with filter fragments
    BE->>CLI: Client.esearch(... use_history=false)
    CLI->>CLI: build EsearchRequest {db, term, retstart, retmax, retmode:"json"}
    CLI->>NCBI: GET esearch.fcgi?db=pubmed&term=(crispr)&retmode=json…
    NCBI-->>CLI: JSON {count, idlist, querytranslation}
    CLI-->>BE: EsearchResult {count, ids, querytranslation, web_env:None, query_key:None}

    BE->>CLI: Client.esummary("pubmed", ids)
    CLI->>CLI: build EsummaryRequest {db, id:"id1,id2,…", retmode:"json"}
    CLI->>NCBI: GET esummary.fcgi?db=pubmed&id=…&retmode=json
    NCBI-->>CLI: JSON {result:{uids, [pmid]:{title, authors, source, …}}}
    CLI->>CLI: for each uid → Summary {…, abstract_text:None}
    CLI-->>BE: Vec<Summary>

    BE-->>API: SearchResponse {count, page, results:[Summary], details:None, elapsed_ms}
    API-->>FE: typed JSON
    FE->>User: render result rows (title + authors + journal, NO abstract)
```

**Key points:**
* Two NCBI calls per search.
* esummary returns metadata only; `abstract_text` on every `Summary`
  is `None`.
* `SearchResponse.details` is omitted from the wire (skip_serializing_if).
* Process-local article cache (`state.articles`) is **not touched**.

---

## 2. Bulk search (`bulk=true`)

Two upstream NCBI hops too — but the second one uses NCBI's history
server (`WebEnv`/`QueryKey`) to pull **full article records in one
shot** instead of just metadata. The result is heavier per call but
every PMID on this page lands in the backend cache.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant FE as FE / App.tsx
    participant API as FE / lib/api.ts
    participant BE as BE / http/search.rs
    participant CLI as BE / infra/ncbi
    participant XML as BE / infra/ncbi/xml.rs
    participant CACHE as BE / infra/cache
    participant NCBI as NCBI

    User->>FE: flip FETCH MODE → Bulk, type "crispr", click Search
    FE->>FE: applySearch() — write URL ?q=crispr&page=1&bulk=1
    FE->>FE: useEffect on URL change
    FE->>API: search({…, bulk:true})
    API->>BE: GET /api/search?term=crispr&bulk=true&page=1&page_size=20

    BE->>CLI: Client.esearch(... use_history=true)
    CLI->>CLI: EsearchRequest with usehistory:"y"
    CLI->>NCBI: GET esearch.fcgi?…&usehistory=y
    NCBI-->>CLI: JSON {count, idlist, querytranslation, webenv, querykey}
    CLI-->>BE: EsearchResult {…, web_env:Some, query_key:Some}

    BE->>CLI: Client.efetch_bulk(web_env, query_key, 0, page_size)
    CLI->>CLI: EfetchRequest {db, id:None, WebEnv, query_key, retstart, retmax, retmode:"xml", rettype:"abstract"}
    CLI->>NCBI: GET efetch.fcgi?…&WebEnv=…&query_key=1&retstart=0&retmax=20
    NCBI-->>CLI: XML <PubmedArticleSet> (N records)
    CLI->>XML: parse_pubmed_xml_bulk(xml)
    XML->>XML: streaming quick-xml walker, per-record ArticleBuilder
    XML-->>CLI: Vec<ArticleDetail>
    CLI-->>BE: Vec<ArticleDetail>

    BE->>CACHE: state.articles.put_many(details.iter().cloned())
    Note over CACHE: HashMap<PMID, ArticleDetail><br/>warmed for this page
    BE->>BE: summary_from_detail(d) → Summary {…, abstract_text:Some(d.abstract_text)}

    BE-->>API: SearchResponse {…, results:[Summary with abstract], details:Some(Vec<ArticleDetail>)}
    API-->>FE: typed JSON
    FE->>User: render result rows WITH inline abstract snippet
```

**Key points:**
* Same number of NCBI hops as Default (2), but the second one is
  heavier — `efetch` returns full XML for all PMIDs on the page.
* `parse_pubmed_xml_bulk` walks `<PubmedArticleSet>` and yields one
  `ArticleDetail` per `<PubmedArticle>` using a per-record
  `ArticleBuilder` to scope state.
* **`state.articles.put_many(...)` is where the speedup lands** — it
  warms the process-local cache so any later `/api/article/{pmid}`
  for these PMIDs is served from memory.
* `summary_from_detail` carries `abstract_text` over from the
  `ArticleDetail` so the frontend can render inline snippets.

---

## 3. Default vs Bulk at a glance

|                           | Default                       | Bulk                                  |
|---------------------------|-------------------------------|---------------------------------------|
| NCBI calls per search     | 2 (esearch + esummary)        | 2 (esearch + efetch_bulk)             |
| Wire format of 2nd call   | JSON, light                   | XML, heavy                            |
| Page payload size         | small                         | ~10×                                  |
| Initial search latency    | ~1.0 s                        | ~1.5–2.0 s                            |
| Abstract in result rows   | no                            | yes (`Summary.abstract_text`)         |
| `state.articles` warmed   | no                            | yes — every PMID on the page          |
| `details` field returned  | `None` (omitted from JSON)    | `Some(Vec<ArticleDetail>)`            |
| Same data per PMID        | guaranteed (see `tests/parity.rs`)                              ||
