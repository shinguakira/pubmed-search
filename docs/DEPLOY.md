# Deployment — Azure Web App for Containers

Single-container deployment. The Axum backend serves both `/api/*` and
the bundled Vite frontend (`STATIC_DIR=/app/dist`), so there's no CORS
to coordinate and no separate frontend hosting to manage.

## Architecture

```
       ┌──────────────────────────────┐
       │      ghcr.io/<owner>/        │
       │       pubmed-search          │  ← image, built in CI
       └──────────────┬───────────────┘
                      │ pulls on restart / new tag
                      ▼
   ┌──────────────────────────────────────┐
   │   Azure Web App for Containers       │
   │   /api/*    → Axum handlers          │
   │   /docs     → Swagger UI             │
   │   anything else → React SPA (dist/)  │
   └──────────────────────────────────────┘
                      │
                      ▼
              eutils.ncbi.nlm.nih.gov
```

## One-time setup

### 1. Confirm the image builds in CI

Push to `main`. The `docker-publish` workflow (under
`.github/workflows/`) builds the multi-stage [Dockerfile](../Dockerfile)
and publishes it to GitHub Container Registry as
`ghcr.io/<owner>/<repo>:latest` and `:sha-<7chars>`.

Confirm in **GitHub → Packages**: the package should appear under the
repo. By default GHCR makes new packages **private** — you can either:

- **Make it public** (Packages → package settings → Change visibility),
  then Azure can pull it without credentials. Recommended for a PoC.
- **Keep it private** and give Azure a PAT — see step 3 below.

### 2. Create the Web App

In Azure Portal:

1. **Create a resource → Web App**.
2. Name: `pubmed-search` (or any globally-unique name; URL becomes
   `https://<name>.azurewebsites.net`).
3. Publish: **Container**.
4. OS: **Linux**.
5. Region: **Japan East** (closest to the dev environment).
6. Pricing plan: **B1 Basic (~¥1,800/mo)** is fine for a PoC. The free
   **F1** tier also works but **does not support custom containers from
   GHCR** — only the platform's built-in stacks. Use B1.
7. On the **Container** tab:
   - Image Source: **Other container registries**
   - Server URL: `https://ghcr.io`
   - Image and tag: `<owner>/<repo>:latest` (lowercased — Azure rejects
     uppercase image names)
   - If the GHCR package is **public**, leave username/password blank.

Click **Review + create → Create**. First-time provisioning takes ~1
minute; the container pull adds ~30–60 s on top.

### 3. (only if the GHCR package is private) Wire credentials

```powershell
# Create a fine-scoped Personal Access Token first:
# GitHub → Settings → Developer settings → Personal access tokens (classic)
# Scope: read:packages only.

az webapp config container set `
  --name pubmed-search `
  --resource-group <rg-name> `
  --docker-custom-image-name ghcr.io/<owner>/<repo>:latest `
  --docker-registry-server-url https://ghcr.io `
  --docker-registry-server-user <github-username> `
  --docker-registry-server-password <PAT>
```

### 4. Set required app settings

The container expects `PORT` and `STATIC_DIR` to be set. The Dockerfile
defaults already cover both, but Azure Web App overrides `PORT` — make
sure these app settings exist:

```powershell
az webapp config appsettings set `
  --name pubmed-search `
  --resource-group <rg-name> `
  --settings `
    WEBSITES_PORT=8080 `
    STATIC_DIR=/app/dist `
    RUST_LOG=pubmed_backend=info,tower_http=info
```

`WEBSITES_PORT` tells the platform which port inside the container to
proxy 443 → ; if you change `PORT` in the Dockerfile, change both.

### 5. Verify

Hit the deployed URL:

```powershell
curl https://pubmed-search.azurewebsites.net/api/health   # → ok
curl -I https://pubmed-search.azurewebsites.net/          # → 200, text/html
```

Then open it in a browser. Search for `crispr`, click a result, watch
the carrier-bird animation deliver an article modal over real NCBI
data.

## Subsequent deploys

Push to `main`. CI rebuilds the image and tags `:latest`. Azure Web App
re-pulls on next request only if you've enabled **Continuous deployment
(webhook)** in Deployment Center, otherwise restart the app once:

```powershell
az webapp restart --name pubmed-search --resource-group <rg-name>
```

## Falling back to Cloud Run

If Web App misbehaves, the same image runs unchanged on Google Cloud Run:

```powershell
gcloud run deploy pubmed-search `
  --image ghcr.io/<owner>/<repo>:latest `
  --region asia-northeast1 `
  --port 8080 `
  --allow-unauthenticated `
  --set-env-vars STATIC_DIR=/app/dist
```

Same on Fly.io:

```powershell
flyctl launch --image ghcr.io/<owner>/<repo>:latest --region nrt
flyctl secrets set STATIC_DIR=/app/dist
```

The Dockerfile is the contract — nothing platform-specific lives outside it.

## Cost (rough)

| Platform                  | This app idling                      |
|---------------------------|--------------------------------------|
| Azure Web App **B1**      | ~¥1,800 / month (~$13)               |
| Azure Web App **F1 free** | does not support GHCR custom images  |
| Cloud Run (free tier)     | ~¥0 — 2M req + 360k GiB-s free       |
| Fly.io (free tier)        | ~¥0 — 3 × 256MB VMs covered          |

Tier-up only when you outgrow B1 (the PoC won't).

## Troubleshooting

- **Web App shows the default Azure splash** — image pull failed.
  Check **Log Stream** for `manifest unknown` (wrong tag), `denied`
  (private package + missing PAT), or `port 8080` mismatches.
- **Container starts but `/api/*` returns 404** — confirm
  `WEBSITES_PORT=8080` is set.
- **Static files served but `/api/search` is slow / fails** — the
  backend needs egress to `eutils.ncbi.nlm.nih.gov:443`. App Service
  allows this by default; no extra NSG / firewall rule needed.
