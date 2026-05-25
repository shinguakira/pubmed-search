# NCBI E-utilities — cheat sheet

The PubMed backend talks to **NCBI E-utilities** at
`https://eutils.ncbi.nlm.nih.gov/entrez/eutils/`. There are 9 endpoints
in the family; the three this app actually uses are **esearch**,
**esummary**, and **efetch**. A fourth concept, the **History server**
(WebEnv / QueryKey), is a server-side cursor that lets `efetch` pull
many records in one go.

This doc lists each endpoint, what it eats, what it spits out, and the
typical call orders.

---

## 1. The endpoints we use

### `esearch.fcgi` — "query → IDs"

| In | Out |
|---|---|
| `db` (e.g. `pubmed`) | `count` — total matching records |
| `term` — PubMed query syntax | `idlist` — PMIDs for the requested page |
| `retstart`, `retmax` — paging (NCBI caps at 100,000 per call) | `querytranslation` — NCBI's expanded query |
| `sort` — relevance / pub_date / first_author / journal / title | (optional) `webenv`, `querykey` — present only when `usehistory=y` |
| `retmode=json` |  |
| `usehistory=y` *(optional)* — also park the result set on the History server |  |

**No metadata, no abstracts. Only IDs.** This is always the first hop
because nothing else accepts a free-text query.

### `esummary.fcgi` — "IDs → short metadata (JSON)"

| In | Out (per PMID) |
|---|---|
| `db` | `title` |
| `id=p1,p2,…` — comma-joined PMIDs | `authors` (short form, e.g. `Liu Y`) |
| `retmode=json` | `source` — journal abbreviation |
| | `pubdate`, `epubdate`, `volume`, `issue`, `pages` |
| | `articleids[doi]` |
| | `pubtype`, `lang` |

**Includes none of**: abstract, MeSH terms, references, affiliations,
full author names.

Bulk-friendly by nature — one call can carry thousands of PMIDs.

### `efetch.fcgi` — "IDs (or History cursor) → full record (XML)"

| In | Out |
|---|---|
| `db` | One `<PubmedArticleSet>` containing one or more `<PubmedArticle>` |
| **Either** `id=p1,p2,…` (URL-length limited, ~200 PMIDs) | each with: title, structured abstract, full authors + affiliations, journal, MeSH, keywords, references, … |
| **or** `WebEnv` + `query_key` (+ `retstart` / `retmax`, up to 10,000 records) |  |
| `retmode=xml`, `rettype=abstract` |  |

**This is the heavy one.** Includes abstract, references, MeSH,
affiliations. Returns XML — needs parsing.

### History server (`WebEnv` + `QueryKey`)

Not an endpoint, a **server-side cursor**. Created as a side-effect of
`esearch?usehistory=y`. NCBI stores the PMID set under a randomly
generated `WebEnv` token + numeric `query_key`. Subsequent `efetch`
calls reference those instead of stuffing the PMID list into the URL —
this is the only way to pull large batches with `efetch` (up to 10,000
records per call).

You can also explicitly upload your own ID set via `epost` (not used
in this app).

---

## 2. Typical call orders

### A. Lightweight metadata listing
*(what a search results page needs when you don't want abstracts)*

```
esearch (db, term)                  → IDs + count
esummary (db, IDs)                  → list of short metadata records
```

Two JSON round trips. No XML. Per-PMID payload is small. **No abstract.**

### B. Single article detail
*(what an article-detail page needs when the user clicked a result)*

```
efetch (db, id=PMID, retmode=xml)   → one full <PubmedArticle>
```

One XML round trip. Yields abstract + everything else.

### C. Full data for many articles in one call
*(what bulk-warming a cache needs)*

```
esearch (db, term, usehistory=y)    → IDs + WebEnv + QueryKey
efetch (db, WebEnv, query_key,
        retstart, retmax,
        retmode=xml)                → <PubmedArticleSet> of up to retmax full records
```

Two round trips total — and the second one pulls up to 10,000 records
without per-PMID overhead. Pattern C is *the* reason the History
server exists.

---

## 3. The other 6 E-utilities (not used by this app, for reference)

| Endpoint | Purpose |
|---|---|
| `epost.fcgi` | Upload an ID list you got from elsewhere onto the History server (returns `WebEnv` + `QueryKey`). Useful when you have IDs but didn't get them via esearch. |
| `elink.fcgi` | Cross-link records (e.g. "what cites this PMID?", "related articles", "same MeSH"). |
| `einfo.fcgi` | Database metadata (last update, indexed field list, total record count). |
| `egquery.fcgi` | Cross-database hit counts for a single term — "how many records match `crispr` in each of PubMed / PMC / Nucleotide / Protein / …". |
| `espell.fcgi` | Spelling suggestions for a query term. |
| `ecitmatch.fcgi` | Resolve free-form citations (journal/volume/page) into PMIDs. |

---

## 4. House rules

* **Rate limit**: 3 req/s anonymous, 10 req/s with `NCBI_API_KEY` set.
* **Every request must include** `tool` + `email` (NCBI's contact-if-
  abusive policy). The backend's `EutilsIdent` struct flattens these
  into every outgoing query string.
* **Large jobs** (thousands of records / minute) should run on weekends
  or 21:00–05:00 US Eastern per NCBI guidance.
