# Data flow: default vs bulk

Two end-to-end traces of what happens when the user runs a search and
then opens one of the resulting articles. Same UI, same DTOs — the only
toggle is `FETCH MODE` (Default / Bulk) under the search input.

Layer prefixes used throughout:
* **[FE]** — React frontend (`frontend/src/...`)
* **[BE]** — Rust backend (`backend/src/...`)
* **[NCBI]** — public NCBI E-utilities endpoint

---

## 1. Default flow (esearch + esummary)

The cheap, lightweight path. Two upstream NCBI calls per search; the
article detail page later costs a third NCBI call for the abstract.

### 1.1 Search

1. **[FE]** User types `crispr` in [SearchBar](../frontend/src/components/SearchBar.tsx) and clicks **Search** (or hits Enter).
2. **[FE]** `SearchBar.onSubmit("crispr")` → calls `applySearch("crispr")` in [App.tsx](../frontend/src/App.tsx).
3. **[FE]** `applySearch` writes to the URL via `setParam`: `q=crispr`, `page=1`, no `bulk` param, plus any pending sort / pageSize / filters.
4. **[FE]** URL change → the fetch `useEffect` whose dep is `fetchKey` fires.
5. **[FE]** [lib/api.ts](../frontend/src/lib/api.ts) `search({...})` issues `GET http://127.0.0.1:8787/api/search?term=crispr&page=1&page_size=20&sort=relevance`.
6. **[BE]** [http/search.rs](../backend/src/http/search.rs) `search` handler runs. `q.bulk` is `false`, so it takes the **default branch**.
7. **[BE]** Combines `term` with comma-separated `?filters=` fragments as `(term) AND filter1 AND filter2 …`. Computes `retstart = (page - 1) * page_size`.
8. **[BE]** Calls `state.ncbi.esearch("pubmed", &term, retstart, page_size, sort, /*use_history=*/false)`. Inside [infra/ncbi/esearch.rs](../backend/src/infra/ncbi/esearch.rs):
   1. Builds an [EsearchRequest](../backend/src/infra/ncbi/dto/request/esearch.rs) with `db=pubmed`, `term`, `retstart`, `retmax`, `retmode=json`, optional `sort`, `usehistory=None`, and the flattened `EutilsIdent` (`tool` / `email` / optional `api_key`).
   2. **[NCBI]** `GET https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term=(crispr)&retstart=0&retmax=20&retmode=json&...`.
   3. Receives JSON `{"esearchresult": {"count": "67459", "idlist": ["25315507", ...], "querytranslation": "...", ...}}`.
   4. Parses it into [EsearchResult](../backend/src/infra/ncbi/dto/response/esearch.rs) (`web_env` / `query_key` stay `None`).
9. **[BE]** Calls `state.ncbi.esummary("pubmed", &es.ids)`. Inside [infra/ncbi/esummary.rs](../backend/src/infra/ncbi/esummary.rs):
   1. If `ids` is empty, returns `vec![]` without an NCBI hop.
   2. Builds an [EsummaryRequest](../backend/src/infra/ncbi/dto/request/esummary.rs) with `db=pubmed`, `id="25315507,27699445,..."`, `retmode=json`, `EutilsIdent`.
   3. **[NCBI]** `GET .../esummary.fcgi?db=pubmed&id=25315507,...&retmode=json&...`.
   4. For each PMID in `result.uids`, plucks the fields we surface (title, authors[].name, source, pubdate, volume, issue, pages, articleids[doi], pubtype, lang) and pushes a [Summary](../backend/src/infra/ncbi/dto/response/summary.rs) with `abstract_text: None` (esummary does not return abstracts).
10. **[BE]** Builds [SearchResponse](../backend/src/http/dto/search.rs) `{ count, page, page_size, query_translation, elapsed_ms, results, details: None }` and returns it as JSON.
11. **[FE]** Promise resolves, `setData(res)` fires, React re-renders. Each row is a [ResultItem](../frontend/src/components/ResultItem.tsx) showing title + authors + journal — **no abstract**, because `summary.abstract_text` is undefined.
12. **[BE article-cache]** `state.articles` (process-local `HashMap<PMID, ArticleDetail>` in [infra/cache/mod.rs](../backend/src/infra/cache/mod.rs)) was **not touched** — the default path never populates it.

### 1.2 Article click

13. **[FE]** User clicks a result title → `react-router-dom` navigates to `/article/25315507`.
14. **[FE]** [ArticlePage.tsx](../frontend/src/pages/ArticlePage.tsx) mounts. `useEffect` calls `getArticle("25315507")` → `GET /api/article/25315507`.
15. **[BE]** [http/article.rs](../backend/src/http/article.rs) handler runs.
16. **[BE]** Checks `state.articles.get("25315507")` → **MISS** (default flow never warmed the cache).
17. **[BE]** Calls `state.ncbi.efetch_abstract("25315507")`. Inside [infra/ncbi/efetch.rs](../backend/src/infra/ncbi/efetch.rs):
    1. Builds an [EfetchRequest](../backend/src/infra/ncbi/dto/request/efetch.rs) with `db=pubmed`, `id=Some("25315507")`, `retmode=xml`, `rettype=abstract`, `web_env=None`, `query_key=None`, `retstart=None`, `retmax=None`, `EutilsIdent`.
    2. **[NCBI]** `GET .../efetch.fcgi?db=pubmed&id=25315507&retmode=xml&rettype=abstract&...`.
    3. Receives PubmedArticleSet XML containing one `<PubmedArticle>`.
    4. Calls `parse_pubmed_xml(xml, pmid)` in [infra/ncbi/xml.rs](../backend/src/infra/ncbi/xml.rs), which delegates to `parse_pubmed_xml_bulk` and picks the matching PMID:
       1. Streams events with `quick-xml::Reader`.
       2. On `<PubmedArticle>` Start: pushes a fresh `ArticleBuilder` (covers title, abstract sections, authors, journal, dates, DOI, keywords, MeSH, pubtypes, references).
       3. Routes text events by the top-of-stack element name (e.g. `ArticleTitle` → title, `AbstractText` → (label, text), `LastName` → current author's last name, `DescriptorName` inside `MeshHeading` → mesh, …).
       4. On `<PubmedArticle>` End: calls `builder.build()` (joins multi-section abstract with `\n\n`, glues YYYY MM DD into a single `pubdate` string) and pushes the resulting [ArticleDetail](../backend/src/infra/ncbi/dto/response/article.rs) to the output `Vec`.
18. **[BE]** Stores the result: `state.articles.put(detail.clone())` so a repeat click is cached.
19. **[BE]** Returns the `ArticleDetail` as JSON.
20. **[FE]** ArticlePage renders title, abstract (paragraph per `\n\n`-separated section), authors, affiliations, keywords, MeSH terms, and References.

### 1.3 Cost summary (default)

| Step | NCBI calls | Approximate wall time |
|------|-----------:|----------------------:|
| Search | 2 (esearch + esummary) | ~1.0 s |
| Each subsequent article click (cold) | 1 (efetch) | ~600 ms |
| Each subsequent article click (warm) | 0 (backend cache) | ~ µs |

---

## 2. Bulk flow (esearch with WebEnv + efetch_bulk)

Heavier upfront, but populates the backend article cache so every
follow-up article click is served from memory.

### 2.1 Search

1. **[FE]** User flips `FETCH MODE` → **Bulk** in [SearchBar](../frontend/src/components/SearchBar.tsx) (this writes to `pendingBulk` only — no fetch yet).
2. **[FE]** User clicks **Search**. `applySearch` writes the URL with `bulk=1` (plus the rest).
3. **[FE]** URL change → fetch `useEffect` fires → `search({..., bulk: true})`.
4. **[FE]** `GET /api/search?term=crispr&page=1&page_size=20&sort=relevance&bulk=true`.
5. **[BE]** [http/search.rs](../backend/src/http/search.rs) `search` handler runs. `q.bulk == true`, so it takes the **bulk branch**.
6. **[BE]** Calls `state.ncbi.esearch(..., /*use_history=*/true)`:
   1. [EsearchRequest](../backend/src/infra/ncbi/dto/request/esearch.rs) is identical to the default path **but** sets `usehistory=Some("y")`.
   2. **[NCBI]** `GET .../esearch.fcgi?...&usehistory=y`.
   3. Response JSON now also carries `"webenv"` (e.g. `MCID_67...`) and `"querykey"` (small integer). The whole result set is now parked on NCBI's history server.
   4. `EsearchResult.web_env = Some(...)`, `query_key = Some(...)`.
7. **[BE]** If `es.ids` is empty, skips step 8 with an empty `details` `Vec`. Otherwise calls `state.ncbi.efetch_bulk(&web_env, query_key, /*retstart=*/0, /*retmax=*/page_size)`. Inside [infra/ncbi/efetch.rs](../backend/src/infra/ncbi/efetch.rs):
   1. Builds [EfetchRequest](../backend/src/infra/ncbi/dto/request/efetch.rs) with `db=pubmed`, **`id=None`** (we point to the history server instead), `retmode=xml`, `rettype=abstract`, `web_env=Some(...)`, `query_key=Some(...)`, `retstart=Some(0)`, `retmax=Some(20)`, `EutilsIdent`.
   2. **[NCBI]** `GET .../efetch.fcgi?db=pubmed&WebEnv=MCID_67...&query_key=1&retstart=0&retmax=20&retmode=xml&rettype=abstract&...`.
   3. Receives a single XML payload containing the full page's worth of `<PubmedArticle>` records (~20 in this example, up to 10 000 NCBI-side).
   4. Calls `parse_pubmed_xml_bulk(xml)` — same walker as the default path, but returns the **whole** `Vec<ArticleDetail>` instead of plucking one PMID.
8. **[BE article-cache]** `state.articles.put_many(details.iter().cloned())` — every PMID on this page lands in the in-memory cache. This is the line where the bulk speedup is staged.
9. **[BE]** Maps each `ArticleDetail` down to a `Summary` via `summary_from_detail(...)`:
   * carries pmid/title/journal/pubdate/doi/pubtypes through,
   * derives `Last F` short author form from `Author.last_name` + initials,
   * leaves esummary-only fields (`epubdate`, `volume`, `issue`, `pages`, `lang`) empty,
   * sets **`abstract_text: Some(d.abstract_text.clone())`** — this is what surfaces as the inline snippet on result rows.
10. **[BE]** Returns [SearchResponse](../backend/src/http/dto/search.rs) `{ count, page, page_size, query_translation, elapsed_ms, results, details: Some(Vec<ArticleDetail>) }`.
11. **[FE]** Promise resolves, `setData(res)` fires, results render. Each [ResultItem](../frontend/src/components/ResultItem.tsx) sees `item.abstract_text` populated and renders the 3-line abstract snippet under the journal line. The frontend ignores `details` — it's purely informational on the wire and the backend already used it for the cache.

### 2.2 Article click

12. **[FE]** User clicks a result title → `/article/25315507`.
13. **[FE]** [ArticlePage.tsx](../frontend/src/pages/ArticlePage.tsx) `useEffect` → `getArticle("25315507")` → `GET /api/article/25315507`.
14. **[BE]** [http/article.rs](../backend/src/http/article.rs) handler runs.
15. **[BE]** Checks `state.articles.get("25315507")` → **HIT** (warmed during step 8).
16. **[BE]** Returns the cached `ArticleDetail` immediately. **No NCBI hop**, no XML parse — just `.clone()` and JSON serialize.
17. **[FE]** ArticlePage renders.

### 2.3 Cost summary (bulk)

| Step | NCBI calls | Approximate wall time |
|------|-----------:|----------------------:|
| Search | 2 (esearch + **efetch_bulk** of 20 records) | ~1.5–2.0 s |
| Each article click on any of those 20 PMIDs | 0 (backend cache hit) | ~ ms (~µs server-side) |
| Article click on a PMID not in the cache | 1 (efetch, then cached) | ~600 ms first time |

---

## 3. Where each flow's speedup actually lands

* **Default**: shape-optimised for the search itself. Cheaper bytes
  (esummary JSON is much smaller than efetch XML) and no extra parse
  work. Best when the user is going to skim title-only.
* **Bulk**: shape-optimised for a "search → open several articles"
  pattern. The first hop is slower because it pulls full abstracts and
  references in XML, but every subsequent article click within the
  page is free. Also surfaces inline abstract snippets on the result
  rows.

Both paths return the **same** `ArticleDetail` shape for a given PMID
— verified by [backend/tests/parity.rs](../backend/tests/parity.rs)
and [backend/tests/fixtures.rs](../backend/tests/fixtures.rs).

The difference is solely **where the speedup is paid**: per-click in
default mode, upfront in bulk mode.
