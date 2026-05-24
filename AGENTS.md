# AGENTS.md

A primer for coding agents (Claude Code, Cursor, Aider, …) and for human
collaborators dropping into the repo for the first time. Read this once
before changing anything; it covers what the project is, how it's wired,
and the conventions to keep when extending it.

## 1. Intent

This is **a personal reading shell on top of PubMed**, not a PubMed
replacement. It's a small full-stack PoC that exists to:

1. Surface NCBI biomedical citations in a calmer, more readable layout
   than the real PubMed UI (full width, tighter rows, newspaper styling).
2. Demonstrate a clean Rust + React stack: thin Rust API in front of a
   public scientific API, modern frontend on top, strict OpenAPI contract
   between them.
3. Be approachable to engineers who don't write Rust day-to-day. Hence
   the heavy doc comments in `backend/src/*.rs` and this file.

It is **not**:

- A scraper. All traffic goes through NCBI E-utilities, the official API.
- An account/auth/persistence layer. Saved articles live in localStorage.
- Production-hardened. CORS is open, no rate limit, no caching.

## 2. Architecture

Three tiers, single direction.

**Frontend** (Vite dev server, port 5173)
- React 18 + TypeScript + Tailwind + shadcn primitives.
- Holds search/article UI, citation dialog, saved list (localStorage only).
- URL params (`q`, `page`, `sort`, `ps`, `display`) are the source of truth so
  searches are shareable.
- Speaks only to the Rust backend over HTTPS+JSON. Never reaches NCBI directly.

**Backend** (Rust + Axum, port 8787)
- Thin proxy in front of NCBI E-utilities. No DB, no cache, no auth.
- HTTP surface: `GET /api/search`, `GET /api/article/{pmid}`,
  `GET /api/cite/{pmid}`, `GET /api/mesh`, `GET /api/health`.
- Doc surface: Swagger UI at `/docs`, raw spec at `/api/openapi.json`. Both are
  generated from `#[utoipa::path]` annotations on the handlers — no hand-
  maintained YAML.
- CORS is wide open in dev. Single shared `pubmed::Client` is injected into
  every handler through Axum's `State` extractor.

**NCBI E-utilities** (`eutils.ncbi.nlm.nih.gov`, public)
- `esearch.fcgi` → IDs + total count (JSON).
- `esummary.fcgi` → batch metadata for IDs (JSON).
- `efetch.fcgi`  → single-record full text (XML; we parse it streaming).
- Rate limit: 3 req/s anonymous, 10 req/s with `NCBI_API_KEY` set.

Request lifecycle for `/api/search?term=crispr`:
1. Frontend issues a fetch; react-query owns the result + caching.
2. Backend `routes::search` joins the term with sidebar filters, calls
   `esearch` then `esummary`, measures wall-clock time via `Instant::now`,
   serializes `SearchResponse` (`elapsed_ms` included).
3. Frontend renders the article rows and shows the elapsed time in the
   toolbar.

**Generated spec lives in two places, intentionally.** The HTTP server
exposes it live at `/api/openapi.json`, and `docs/openapi.json` is a
committed copy regenerated via `npm run openapi`. The committed copy is
what reviewers diff against in PRs.

Key invariants:

- **Frontend never calls NCBI directly.** Always through the Rust backend
  so we can centralize API keys, rate concerns, CORS, and shape conversion
  (NCBI XML → typed JSON).
- **Every handler is in the OpenAPI spec.** Enforced at compile time via
  `utoipa-axum`'s `routes!(handler)` macro (`backend/src/lib.rs`). Adding
  a route without `#[utoipa::path(...)]` is a build error.
- **Tests hit live NCBI, no mocks.** Both the Rust integration tests
  (`backend/tests/api.rs`) and the Playwright suite (`e2e/`) talk to the
  real backend talking to the real NCBI. Tests are serialized + throttled
  to stay under the 3 req/s anonymous limit.

## 3. File map

### Repository root

| Path | What it is |
|------|------------|
| `package.json`          | npm workspace root. Provides `dev`, `build`, `e2e`, `test:backend`, `openapi`. |
| `playwright.config.ts`  | Spawns both backend + frontend via `webServer`, runs Chromium specs in `e2e/`. |
| `docs/openapi.json`     | Generated OpenAPI 3.1 spec, committed for review. Regen with `npm run openapi`. |
| `docs/screenshot.png`   | Edge-headless screenshot of the search page used in the README. |
| `e2e/*.spec.ts`         | Playwright tests, no mocks. Hit live NCBI through the real stack. |
| `README.md`             | User-facing intro + run instructions. |
| `AGENTS.md`             | (This file.) Architecture + conventions for agents/contributors. |

### Backend (`backend/`)

| Path | Role |
|------|------|
| `Cargo.toml`            | Crate manifest; axum 0.7, utoipa 5, utoipa-axum 0.1, utoipa-swagger-ui 8. |
| `src/main.rs`           | Tiny binary: logger init, bind 127.0.0.1:8787, `axum::serve(app())`. |
| `src/lib.rs`            | Top-level wiring (`app()`, `openapi()`); re-exports `http::ApiDoc`. |
| `src/state.rs`          | `AppState` — shared deps (`ncbi` client, future DB pool) injected into handlers. |
| `src/error.rs`          | `AppError` enum + `ErrorResponse` body + `IntoResponse` (single HTTP mapping). |
| `src/http/mod.rs`       | `ApiDoc` + `build(state)` → `(Router, OpenApi)` via `OpenApiRouter::routes!()`. |
| `src/http/{search,article,mesh,cite}.rs` | One handler per file, owns its `*Query`/`*Response` DTOs + `#[utoipa::path]`. |
| `src/infra/ncbi/`       | NCBI client split per endpoint (`client.rs`, `esearch.rs`, `esummary.rs`, `efetch.rs`, `xml.rs`, `types.rs`). |
| `src/infra/db/`         | (Empty placeholder.) Future: sqlx pool + repository modules. |
| `src/domain/`           | (Empty placeholder.) Future: pure logic (citation generation, search-query construction) extracted from `http/` for unit testing without IO. |
| `src/bin/gen-openapi.rs`| `cargo run --bin gen-openapi` → writes `docs/openapi.json`. |
| `tests/api.rs`          | 4 live-NCBI integration tests; serialized with `OnceLock<Mutex>` + 400 ms throttle. |
| `tests/openapi.rs`      | 3 offline spec-shape tests: every expected path exists, `SearchResponse.elapsed_ms` present, every op has a 500. |

### Frontend (`frontend/`)

| Path | Role |
|------|------|
| `package.json`          | React 18 + Vite 5 + Tailwind 3 + shadcn primitives + react-query + react-router. |
| `vite.config.ts`        | Binds `127.0.0.1` (avoids Node 18+ IPv6 issue with Playwright). No proxy. |
| `tailwind.config.js`    | Newspaper palette (`paper.*` colors) on top of shadcn HSL CSS vars. |
| `src/App.tsx`           | Search page: URL state + filter sidebar + results panel. |
| `src/pages/ArticlePage.tsx` | Article detail (abstract, MeSH, authors, affiliations). |
| `src/components/Header.tsx` | "The PubMed Gazette" masthead. |
| `src/components/SearchBar.tsx` | Main search input + opens the Advanced builder. |
| `src/components/AdvancedBuilder.tsx` | AND/OR/NOT × field-tag rows with live query preview. |
| `src/components/FiltersSidebar.tsx`  | Publication date / text availability / article attribute / type / language / species / sex / age / other. |
| `src/components/ResultsToolbar.tsx`  | "67,458 dispatches · 0.42s" + display/sort/per-page selects. |
| `src/components/ResultItem.tsx`      | One newspaper-styled article row. |
| `src/components/Pagination.tsx`      | First / prev / next / last. |
| `src/components/CiteDialog.tsx`      | AMA / APA / MLA / NLM / BibTeX tabs with one-click copy. |
| `src/components/SavedDialog.tsx`     | LocalStorage-backed bookmarks + JSON export. |
| `src/components/ui/*.tsx`            | Vendored shadcn primitives (button, dialog, …). |
| `src/lib/api.ts`        | Typed client. Calls `${VITE_API_URL ?? "http://127.0.0.1:8787"}/api/...`. |
| `src/hooks/useSaved.ts` | LocalStorage hook + cross-tab sync via a custom event. |

## 4. Conventions

### Backend

- **Handlers always carry `#[utoipa::path(...)]`**. The `routes!()` macro
  in `lib.rs` makes the build fail otherwise. Use existing handlers as
  templates: `tag`, `params(...)` from a struct that derives `IntoParams`
  or inline `("name" = Type, Path/Query, description = "...")`, and
  responses for at least 200 and 500.
- **Response/request DTOs derive `ToSchema`** so they show up under
  `#/components/schemas/...`. If a DTO isn't in the schema, double-check
  the derive.
- **Errors funnel through `AppError`** (`error.rs`). Anything `Into<
  anyhow::Error>` is caught by `?`. Surface a 500 with `ErrorResponse`.
  `NotFound` and `BadRequest(String)` variants are reserved for the DB
  layer and richer validation respectively.
- **NCBI traffic lives only in `pubmed::Client`**. Don't call NCBI from
  handlers directly — add a method on `Client` and call that.
- **No mocks in tests.** New integration tests go in `backend/tests/`,
  acquire the `ncbi_gate()` mutex, sleep 400 ms, then exercise the real
  app via reqwest.
- After changing a handler signature or DTO, regenerate the spec:
  `npm run openapi`, commit the diff in `docs/openapi.json`.

### Frontend

- **No Vite dev proxy.** The frontend calls `http://127.0.0.1:8787` directly;
  backend CORS allows any origin in dev. Override with `VITE_API_URL`.
- **Server state lives in `@tanstack/react-query`**; URL state (`q`, `page`,
  `sort`, `ps`, `display`) is the source of truth so searches are shareable.
- **Tailwind `paper.*` palette** for the newspaper aesthetic. Add new colors
  to `tailwind.config.js` rather than littering arbitrary HSL values.
- **shadcn primitives are vendored under `src/components/ui/`.** Edit them
  freely — they're our code now, not a dependency.

### Tests

- Backend live tests: `npm run test:backend` (or
  `cargo test --manifest-path backend/Cargo.toml --tests -- --nocapture`).
  Each prints wall-clock ms via `eprintln!`.
- OpenAPI shape tests: included in the same `cargo test` run, but they're
  offline and fast.
- E2E: `npm run e2e` (needs `npm run e2e:install` once for Chromium).

## 5. Common tasks

### Add a new endpoint

1. Create a new file under `backend/src/http/` for the resource
   (`http/<name>.rs`).
2. In that file: define `*Query`/`*Path` params (`IntoParams + Deserialize`),
   `*Response` body (`Serialize + ToSchema`), and the handler `async fn`
   with `#[utoipa::path(...)]`.
3. Add `pub mod <name>;` to `backend/src/http/mod.rs`, and register it
   via `.routes(routes!(<name>::<handler>))`. *Build fails here if the
   macro can't find the path attribute — good.*
4. Update `backend/tests/openapi.rs` if you want to assert the path exists.
5. Add a live integration test in `backend/tests/api.rs` (mind the gate).
6. `npm run openapi` to regenerate `docs/openapi.json`. Commit it.
7. Add a typed client method in `frontend/src/lib/api.ts`.
8. Wire UI as needed. Use react-query for caching.

### Add a new filter / search facet

1. Extend the `Filters` type in `frontend/src/components/FiltersSidebar.tsx`
   and add to `SECTIONS` so it renders in the sidebar.
2. Map each option's `value` to a PubMed filter expression (`Review[pt]`,
   `humans[mesh]`, …). The expression is appended to the term server-side.
3. The backend does not need changes — it already takes `?filters=` as a
   comma-separated list of raw PubMed filter expressions.

### Change the newspaper styling

1. Edit `tailwind.config.js` (`paper.*` palette) for color tweaks.
2. Section-level layout lives in `src/App.tsx` (grid template, padding).
3. Per-row styling is in `src/components/ResultItem.tsx`.
4. Verify visually: `npm run dev`, hit `http://localhost:5173/?q=crispr`.

### Regenerate the README screenshot

```powershell
& "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" `
  --headless=new --disable-gpu `
  --screenshot=docs\screenshot.png `
  --window-size=1440,1100 --hide-scrollbars --virtual-time-budget=8000 `
  "http://localhost:5173/?q=crispr"
```

## 6. Things that are easy to get wrong

- **Node 18+ on Windows resolves `localhost` to `::1` first.** Anything
  binding to "localhost" by default (Vite, Node test runners) will fail
  to be reached when something checks `127.0.0.1`. Both `vite.config.ts`
  and `backend/src/main.rs` now bind explicitly to `127.0.0.1`. Don't
  change either back to "localhost" without thinking.
- **Stale `vite.config.js` next to `vite.config.ts`.** Vite picks `.js`
  over `.ts`. The `.gitignore` ignores them; if one appears, delete it.
- **NCBI rate limits.** 3 req/s anonymous, 10 req/s with an API key in
  `NCBI_API_KEY`. Live tests serialize + throttle; parallel test runs
  will get spurious 0-count responses without it.
- **`#[utoipa::path]` path string must match the router path.** Both are
  read from the macro by `routes!()`, so a mismatch is impossible in
  practice — but if you copy-paste a handler and forget to change the
  path, you'll register two handlers on the same URL. The router will
  panic at startup.
- **Cite formats are hand-rolled.** AMA/APA/MLA/NLM/BibTeX in
  `routes::cite` are template strings, not a real citation library. Good
  enough for the PoC, but don't trust them for a manuscript.

## 7. When the spec changes

Treat `docs/openapi.json` like a database migration:

1. Change the handler / DTO.
2. `npm run openapi` to regenerate.
3. `git diff docs/openapi.json` — sanity check what changed.
4. Update typed client (`frontend/src/lib/api.ts`) to match.
5. Commit handler + spec + client in one go.
