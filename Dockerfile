# syntax=docker/dockerfile:1.6
#
# Single-container production image for the PubMed-search app.
#
#   stage 1 (frontend) — `npm ci -w frontend && npm run build -w frontend`
#                        produces frontend/dist/
#   stage 2 (backend)  — `cargo build --release` produces the Axum binary
#   stage 3 (runtime)  — Debian slim with just the binary + frontend dist
#
# The Axum binary serves `/api/*` plus a `ServeDir` over the bundled
# frontend; non-API requests fall through to `index.html` so React Router
# resolves them client-side. One container, one process, no CORS.

# ───────────────────── stage 1: frontend ─────────────────────
FROM node:20-bookworm-slim AS frontend
WORKDIR /app

# Repo uses npm workspaces — the lockfile lives at the root, not inside
# frontend/. Bring both package.json files in first so the install layer
# caches across source edits.
COPY package.json package-lock.json ./
COPY frontend/package.json frontend/package.json
RUN --mount=type=cache,target=/root/.npm \
    npm ci --workspace frontend --no-audit --no-fund

# Build the static bundle. VITE_API_URL="" forces same-origin relative
# fetches at runtime ("/api/search" instead of "http://127.0.0.1:8787/api/search").
COPY frontend/ frontend/
ENV VITE_API_URL=""
RUN npm run build --workspace frontend

# ───────────────────── stage 2: backend ──────────────────────
FROM rust:1.84-slim-bookworm AS backend
WORKDIR /app/backend
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cache deps separately from sources so source-only edits skip the slow
# first-pass `cargo fetch`.
COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY backend/ ./
RUN cargo build --release --bin pubmed-backend

# ───────────────────── stage 3: runtime ──────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=backend  /app/backend/target/release/pubmed-backend ./pubmed-backend
COPY --from=frontend /app/frontend/dist ./dist

# Tell the Axum router where the bundle lives. Azure Web App / Cloud Run /
# Fly all inject $PORT — main.rs picks that up automatically.
ENV STATIC_DIR=/app/dist \
    RUST_LOG="pubmed_backend=info,tower_http=info" \
    PORT=8080

USER app
EXPOSE 8080
CMD ["./pubmed-backend"]
