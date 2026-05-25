# Why the default vs bulk benchmark exists

This is **not** a "which one wins" race. The two paths exercise two
fundamentally different NCBI E-utilities access patterns; the
benchmark documents their behavior side by side and catches
regressions where one path drifts from the other.

## The two paths

| | default (`bulk=false`) | bulk (`bulk=true`) |
|---|---|---|
| esearch | plain, returns PMIDs | with `usehistory=y`, also parks the set on NCBI's History server (returns `WebEnv` + `query_key`) |
| efetch  | id list packed into POST body (`POST efetch.fcgi` with `id=p1,p2,…`) | server-side cursor reference (`POST efetch.fcgi` with `WebEnv=…&query_key=…`) |

Both end up returning the same XML records for the same query. The
shapes of the requests, the server-side handling, and the
operational envelopes are different.

## What this benchmark is measuring

The comparison axis is **access-pattern cost**:
- How does NCBI handle a packed id list vs a History cursor?
- Does the difference grow / shrink / flip as `N` (records per call)
  scales from a UI page (≲100) to NCBI's per-call ceiling (10,000)?
- Does one pattern degrade differently from the other under server
  load or rate-limit pressure?

The benchmark answers all of those with live numbers, against the
real NCBI service. The terms are varied across selectivities
(crispr cas9, glaucoma, cancer, diabetes, …) to keep one quirky
query from dominating the picture. The `hits` column is reported
purely as context (each term's corpus size); the wall-time
measurement is governed by `N` (records fetched), not `hits`.

## What the benchmark is NOT

- It is **not** a "bulk is faster" demo. If they post the same
  numbers, that itself is a finding — and it should never trigger
  "delete one of them".
- It is **not** a microbenchmark of our parser or our HTTP client.
  Both paths share the same parser (`parse_pubmed_xml_bulk`) and the
  same `reqwest::Client`.
- It is **not** a finished verdict. NCBI's behavior changes over
  time; this baseline is something we re-run when in doubt.

## Operational differences beyond wall time

Things the benchmark does not (currently) measure but are real
distinguishers of the two paths:

- **WebEnv reusability**: the bulk path leaves a server-side cursor
  pointing at the result set, so a follow-up call (different
  `rettype`, paging through with `retstart`, an `elink` chain, …)
  can be issued without re-sending the IDs. The default path has to
  ship the id list every time.
- **Body size growth**: at very large `N` the default path's POST
  body grows linearly with `N` (~9 bytes per PMID). The bulk path's
  body is constant-sized (a short `WebEnv` string + a small integer).
- **Cursor freshness**: NCBI expires History entries after ~24 h of
  inactivity. The default path has no such state.
- **Retry semantics**: a transient failure on the bulk path can
  potentially re-issue the same `efetch` with the same cursor; the
  default path has to know the original id list to retry.

These are why the two paths exist as separate, selectable modes —
not because we expect one to beat the other in wall time.

## Hands-off rules

1. **Do not delete the benchmark.**
2. **Do not collapse the two paths into one**, even if wall-time
   numbers look close.
3. **Do not remove the UI toggle.**
4. When numbers look close, vary `N`, term selectivity, and re-run
   before drawing any conclusion. Never write "they're equivalent".
