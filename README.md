# pubmed-search

PubMed-style biomedical-literature search PoC.

- **Backend**: Rust + Axum — proxies NCBI E-utilities (`esearch`, `esummary`, `efetch`), keeps the API key server-side and avoids CORS issues.
- **Frontend**: Vite + React + TypeScript + TailwindCSS + shadcn/ui — UI layout and search controls mirror PubMed, restyled with shadcn.

## Layout

```
pubmed-search/
├── backend/         Rust + Axum API at http://localhost:8787
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── pubmed.rs   NCBI E-utilities client + XML parser
│       └── routes.rs   /api/search, /api/article/:pmid, /api/mesh, /api/cite/:pmid
└── frontend/        Vite dev server at http://localhost:5173 (calls backend directly via CORS)
    ├── package.json
    └── src/
        ├── App.tsx               Search page
        ├── pages/ArticlePage.tsx Article detail
        ├── components/
        │   ├── Header.tsx
        │   ├── SearchBar.tsx
        │   ├── AdvancedBuilder.tsx
        │   ├── FiltersSidebar.tsx
        │   ├── ResultsToolbar.tsx
        │   ├── ResultItem.tsx
        │   ├── Pagination.tsx
        │   ├── CiteDialog.tsx
        │   ├── SavedDialog.tsx
        │   └── ui/               shadcn primitives
        └── lib/api.ts            Typed client for the backend
```

## Run

One-time install from the repo root (npm workspaces hoists `frontend/` deps):

```powershell
cd E:\workspace\PoC\pubmed-search
npm install
```

### Both at once (recommended)

```powershell
npm run dev    # cargo run + vite dev, color-prefixed via concurrently
```

Stop with `Ctrl-C` once — concurrently shuts down both.

### Individually

```powershell
npm run dev:backend   # http://localhost:8787  (cargo run)
npm run dev:frontend  # http://localhost:5173  (Vite; calls backend directly via CORS)
```

### Optional NCBI credentials (10 req/s instead of 3)

```powershell
$env:NCBI_API_KEY = "your_key_here"
$env:NCBI_EMAIL   = "you@example.com"
npm run dev
```

### Production-ish

```powershell
npm run build   # cargo build --release + vite build
npm run start   # cargo run --release + vite preview
```

## Features

| Area              | Notes                                                                  |
|-------------------|------------------------------------------------------------------------|
| Top search bar    | Single input + Search; matches PubMed entry point                     |
| Advanced builder  | Boolean rows (AND/OR/NOT) + field tags ([ti], [au], [mesh], [dp]…)    |
| Filter sidebar    | Publication date, article type, species, language, sex, age, text     |
| Results list      | Numbered items, citation metadata, PMID/DOI/PubType chips             |
| Sort & paging     | Best match / recent / first author / journal / title, 10/20/50/100   |
| Article detail    | Abstract (structured), authors, affiliations, MeSH, keywords          |
| Cite              | AMA / APA / MLA / NLM / BibTeX with one-click copy                    |
| Save              | LocalStorage-backed, JSON export                                      |
| URL state         | `?q=…&page=…&sort=…&ps=…` — shareable searches                        |

## Notes

- NCBI rate limits: 3 req/s anonymous, 10 req/s with API key. The backend sends `tool`/`email` per NCBI guidelines.
- The Rust backend is intentionally a thin proxy — no DB, no auth. Add Redis/PG later if you want cached searches or accounts.
