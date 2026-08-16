# Deployment & Operations Readiness

> **Reviewer:** Atlas (DevOps/SRE specialist) · **Date:** 2026-08-14
> **Target:** `tickets_api` latest `v2` branch — Dockerfile, docker-compose.yaml, .github/workflows/docker-publish.yml, .cargo/config.toml

## Executive Summary

> **Verdict: Not deployable.** The container will **panic before it ever binds port 8080.** Two independent panic conditions fire in sequence at startup, the compose file omits 4 of 6 required env vars, and destructive dev-init code runs unconditionally. The existing CI pipeline builds and pushes an image guaranteed to crash-loop, with zero Rust-level quality gates upstream.

**Deployment readiness scorecard:**

| Criterion | Status |
|---|---|
| Container builds | Yes (1.2 GB, root) |
| Starts | ❌ No (panics immediately) |
| Config injection | ❌ No (<50% vars) |
| Secrets | ❌ No (exposed) |
| Health probes | ❌ No |
| Graceful shutdown | ❌ No |
| CI quality gates | ❌ No |
| Observability | ⚠️ Insufficient |
| DB persistence | ❌ No |
| Dep reproducibility | ❌ Broken (Cargo.lock gitignored) |
| **Overall** | **0/10** |

---

## 1. Startup Panic Sequence

```mermaid
flowchart TB
  Main["main() → server::startup()"] --> Obs["observability::startup() ✅"]
  Main --> Dev["line 16: _dev_utils::init_dev()"]
  Dev --> DevDb["init_dev_deb() dev_db.rs:24"]
  DevDb --> Pool1["new_db_pool<br/>postgres://postgres:welcome@db:5432/postgres"]
  Pool1 --> Panic1["'db' hostname unresolvable<br/>(service is 'db_test')<br/>→ .unwrap() → PANIC ❌ #1"]
  DevDb --> ReadSql["fs::read_to_string<br/>'sql/dev_initial/...'"]
  ReadSql --> Panic2["SQL files not in runtime image<br/>→ .unwrap() → PANIC ❌ (if #1 bypassed)"]
  Main --> MM["line 17: ModelManager::new().await.unwrap()"]
  MM --> Config["config().DB_URL<br/>→ get_env('SERVICE_DB_URL')"]
  Config --> Panic3["env::var() fails<br/>→ ConfigMissingEnv<br/>→ panic!('FATAL') ❌ #2"]
```

**Both conditions are present simultaneously.** The container cannot start.

---

## 2. Dockerfile Analysis

**File:** `tickets_api/Dockerfile` (19 lines)

```dockerfile
FROM rust:1.76 as build      # full toolchain (build stage)
WORKDIR /app
COPY . .                      # all source + Cargo.toml together
RUN cargo build --release
FROM rust:1.76                # full toolchain (runtime stage!)
WORKDIR /usr/local/bin
COPY --from=build /app/target/release/tickets_api .
EXPOSE 8080
CMD ["./tickets_api"]
```

### Issues

| # | Issue | Severity |
|---|---|---|
| C1 | Runtime image is `rust:1.76` (~1.2 GB) — builder and runtime are the same base image, so multi-stage saves nothing | Critical |
| C2 | Runs as root — no `USER` directive | Critical |
| C3 | No dep-cache layer ordering — `COPY . .` invalidates layer cache on any code change → full recompile (~5 min) | High |
| C4 | `.dockerignore` = `**/target` only — `.git/`, `.cargo/` secrets, `sql/` seeds in context | Medium |
| C5 | No `HEALTHCHECK` | High |
| C6 | `WORKDIR /usr/local/bin` shadows system tools | Low |
| C7 | No `--platform`/multi-arch | Medium |
| C8 | SQL files + `web-folder/` not copied to runtime image — static fallback 404s | High |

### Hardened Dockerfile

```dockerfile
ARG RUST_VERSION=1.76.0
ARG DEBIAN_VERSION=bookworm

# ---- Build stage ----
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS build
WORKDIR /app

# Dependency caching: build a dummy project first
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked 2>/dev/null || true

# Copy real source and build
COPY . .
RUN cargo build --release --locked

# ---- Runtime stage ----
FROM debian:${DEBIAN_VERSION}-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN groupadd --gid 10001 tickets \
    && useradd --uid 10001 --gid tickets --shell /bin/false --create-home tickets
WORKDIR /app
COPY --from=build /app/target/release/tickets_api /app/tickets_api
COPY --from=build /app/web-folder /app/web-folder
RUN chown -R tickets:tickets /app
USER tickets
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:8080/health || exit 1
CMD ["/app/tickets_api"]
```

**Improvements:** runtime 1.2 GB → ~80 MB (-93%); root → UID 10001; dep caching; `--locked`; `HEALTHCHECK`; `/app` workdir; version-pin patch + OS.

---

## 3. docker-compose.yaml Analysis

**File:** `tickets_api/docker-compose.yaml`

### Issues

| # | Issue | Severity |
|---|---|---|
| D1 | `api` service missing 4 of 6 required env vars (only `RUST_LOG` + `SERVICE_WEB_FOLDER` set) | Critical |
| D2 | `.cargo/config.toml [env]` does NOT apply to bare `./tickets_api` binary → config panic | Critical |
| D3 | `init_dev()` panics first — hardcoded `db` hostname doesn't resolve (service named `db_test`) | Critical |
| D4 | No healthcheck on `db_test` — api starts before PG ready | High |
| D5 | No healthcheck on `api` | High |
| D6 | No DB volume — data lost on recreation | High |
| D7 | No named network | Low |
| D8 | No restart policy | High |
| D9 | Hardcoded dev DB password | Medium |
| D10 | `web-folder/` doesn't exist and isn't mounted | Medium |
| D11 | `version: "3.8"` deprecated | Low |
| D12 | SQL password typo `dev_only_pws` vs `dev_only_pwd` | Medium |

### Hardened docker-compose.yaml

```yaml
services:
  api:
    build: .
    ports:
      - "8080:8080"
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped
    env_file: .env
    environment:
      RUST_LOG: "tickets_api=info,sqlx=warn,tower_http=warn,hyper=warn"
      SERVICE_WEB_FOLDER: "/app/web-folder/"
      SERVICE_DB_URL: "postgresql://app_user:${DB_PASSWORD}@db:5432/app_db"
      SERVICE_PWD_KEY: "${SERVICE_PWD_KEY}"
      SERVICE_TOKEN_KEY: "${SERVICE_TOKEN_KEY}"
      SERVICE_TOKEN_DURATION_SEC: "${SERVICE_TOKEN_DURATION_SEC:-1800}"
    volumes:
      - ./web-folder:/app/web-folder:ro
    networks:
      - tickets-net
    healthcheck:
      test: ["CMD", "curl", "--fail", "--silent", "http://127.0.0.1:8080/health"]
      interval: 10s
      timeout: 3s
      retries: 3
      start_period: 5s

  db:
    image: postgres:16-bookworm
    restart: unless-stopped
    ports:
      - "5433:5432"
    environment:
      POSTGRES_USER: app_user
      POSTGRES_PASSWORD: "${DB_PASSWORD}"
      POSTGRES_DB: app_db
    volumes:
      - db_data:/var/lib/postgresql/data
    networks:
      - tickets-net
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app_user -d app_db"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 5s

networks:
  tickets-net:
    driver: bridge

volumes:
  db_data:
```

### Companion `.env` template (gitignored)

```env
# === DO NOT COMMIT THIS FILE ===
DB_PASSWORD=change_me_to_a_real_password
SERVICE_PWD_KEY=replace_with_real_base64url_key
SERVICE_TOKEN_KEY=replace_with_real_base64url_key
SERVICE_TOKEN_DURATION_SEC=1800
RUST_LOG=tickets_api=info,sqlx=warn,tower_http=warn,hyper=warn
```

---

## 4. Environment & Config

### Two non-overlapping config mechanisms

| Mechanism | Applies to | Used by Docker? |
|---|---|---|
| `.cargo/config.toml [env]` | `cargo` subcommands only (`cargo run`, `cargo test`) | ❌ No |
| `std::env::var` in `config/mod.rs` | The running process (bare binary) | ✅ Yes |

The `.cargo/config.toml` file itself acknowledges this (lines 11-14 comment). The Dockerfile launches `./tickets_api` directly, so `[env]` is inert — and the compose file doesn't provide the vars either.

### Secrets committed to git

| Secret | Location | Value (decoded) |
|---|---|---|
| PWD_KEY | `.cargo/config.toml:24` | `dev_only_pwd_key` (17 bytes ASCII) |
| TOKEN_KEY | `.cargo/config.toml:25` | `dev_only_token_key` (18 bytes) |
| DB URL | `.cargo/config.toml:22` | `app_user:dev_only_pwd@localhost:5432` |
| Postgres superuser | `_dev_utils/dev_db.rs:14` | `postgres:welcome@db:5432` |
| SQL password (typo) | `00-recreate-db.sql:14` | `dev_only_pws` |
| Compose password | `docker-compose.yaml:30` | `dev_only_pwd` |

### 12-Factor scorecard

| Factor | Status | Notes |
|---|---|---|
| III Config | ⚠️ Partial | Env-only, but hardcoded in `.cargo/config.toml` |
| IV Backing services | ❌ No | Hardcoded dev URLs |
| IX Disposability | ❌ No | No graceful shutdown; `unwrap` panics |
| XI Logs | ⚠~OK | stdout but no timestamps/structured fields |
| XIII CI/CD | ⚠️ Partial | Image build exists, no test gate |

---

## 5. Observability

### Issues (O1–O7)

| # | Issue | Severity |
|---|---|---|
| O1 | `.without_time()` — no RFC3339 timestamps | High |
| O2 | `.with_target(false)` — no module path | Medium |
| O3 | No structured fields — human text only, no JSON | High |
| O4 | No `#[tracing::instrument]` — zero span context | Medium |
| O5 | No signal handler — SIGTERM kills in-flight requests, open DB transactions | Critical |
| O6 | `.unwrap()` on `axum::serve` — port conflict → panic | High |
| O7 | `.unwrap()` on startup — `ModelManager::new()` / `TicketController::new()` panic on DB failure, no retry | High |

**`RUST_LOG=tickets_api=debug` in prod** → every SQL query param, every HTTP request detail, every cookie operation → massive log volume, potential PII exposure.

**Recommended prod level:** `RUST_LOG=tickets_api=info,sqlx=warn,tower_http=warn,hyper=warn`

**No metrics** (no Prometheus, no `/metrics`, no latency/pool/error tracking).
**No health/readiness endpoints** — `grep health|readiness|liveness = 0 matches`. `GET /` is NOT a health probe (depends on full middleware stack incl auth).

### Fix: observability init

```rust
// Replace .without_time() and .with_target(false) with:
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .json()              // structured fields
    .init();
```

---

## 6. Build & Dependency

| Item | Value | Issue |
|---|---|---|
| Edition | 2021 | OK |
| Rust in Docker | 1.76 | Downlevel (2026 stable ~1.80+) |
| `rust-toolchain.toml` | Missing | Add to pin |
| `Cargo.lock` | **Gitignored** | Binary crate MUST commit it |
| `tokio` features | `full` | Excessive — prefer `rt-multi-thread,macros,net,signal,time` |
| `tower-http` features | `fs` only | Need `cors`, `trace`, `compression`, `timeout`, `limit` |
| `[profile.release]` | Missing | Add `lto="fat"` + `strip=true` → ~8 MB binary |
| `--locked` in Docker | Not used | Add for reproducibility |
| SBOM | Missing | Add `syft`/`cargo cyclonedx` |
| Cosign signing | Exists in CI | Good |
| Digest pinning | Missing | Pin in compose/K8s manifests |

---

## 7. CI/CD

### Existing workflow (`.github/workflows/docker-publish.yml`)

**Strengths:** BuildKit/Buildx multi-platform · push to ghcr.io · keyless cosign signing · GHA build cache · pinned action SHAs · skip-push on PR · `id-token: write`.

### Gaps (G1–G13)

| # | Gap | Fix |
|---|---|---|
| G1 | No `cargo fmt --check` | Add fmt job |
| G2 | No `cargo clippy` | Add clippy job (`-D warnings -W clippy::unwrap_used -W clippy::expect_used`) |
| G3 | No `cargo test` | Add test job with postgres service |
| G4 | No `cargo audit`/`cargo-deny` | Add audit job (`rustsec/audit-check@v2.0.0`) |
| G5 | No standalone `cargo build --release` | Add build job before Docker, use `--locked` |
| G6 | No `cargo doc` | Optional |
| G7 | No SBOM | Add `syft` or `cargo cyclonedx` |
| G8 | No SAST | Add `semgrep` or CodeQL |
| G9 | No secret scanning | Add `trufflehog`/`gitleaks` |
| G10 | No branch protection | Enforce on `main` |
| G11 | `actions/checkout@v3` → v4 | Update |
| G12 | No Dependabot/Renovate | Add config |
| G13 | No test matrix | Consider OS/rust-version matrix |

### Suggested `ci.yml` workflow

```yaml
name: CI
on:
  push:
    branches: [main, v2]
  pull_request:
    branches: [main, v2]

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check
        working-directory: tickets_api

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used
        working-directory: tickets_api

  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: app_user
          POSTGRES_PASSWORD: dev_only_pwd
          POSTGRES_DB: app_db
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 3s
          --health-retries 5
    env:
      SERVICE_DB_URL: postgresql://app_user:dev_only_pwd@localhost:5432/app_db
      SERVICE_PWD_KEY: ZGV2X29ubHlfcHdkX2tleQ==
      SERVICE_TOKEN_KEY: ZGV2X29ubHlfdG9rZW5fa2V5
      SERVICE_TOKEN_DURATION_SEC: "1800"
      SERVICE_WEB_FOLDER: web-folder/
      RUST_LOG: tickets_api=debug
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-targets --all-features
        working-directory: tickets_api

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2.0.0
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --locked
        working-directory: tickets_api
      - uses: actions/upload-artifact@v4
        with:
          name: tickets_api-binary
          path: tickets_api/target/release/tickets_api
```

### Updated `docker-publish.yml` (key changes)

- Add `needs: ci` (require all CI jobs to pass before image build)
- `actions/checkout@v4` (from v3)
- `cosign-installer@v2.2.4` (from v3.1.1/v2.1.1)
- `build-push-action@v6` (from v5.0.0)
- Add `provenance: true` and `sbom: true` to metadata

---

## 8. Code Fixes Required for the Container to Boot

### Fix 1 — Feature-gate dev init

```rust
// server.rs
#[cfg(debug_assertions)]
let _ = _dev_utils::init_dev().await;
// or use a custom feature:
// #[cfg(feature = "dev-utils")]
// let _ = _dev_utils::init_dev().await;
```

### Fix 2 — Remove startup unwraps

```rust
// server.rs — replace .unwrap() with proper error handling
let manager = ModelManager::new().await
    .map_err(|e| {
        tracing::error!("Failed to create ModelManager: {e:?}");
        e
    })?;
```

### Fix 3 — Graceful shutdown

```rust
// server.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Shutdown signal received");
}

// Replace: axum::serve(listener, app).await.unwrap();
// With:
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

### Fix 4 — Health endpoints

```rust
// Add to router
fn health_router() -> axum::Router<ModelManager> {
    axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .route("/ready", axum::routing::get(|State(m): State<ModelManager>| async move {
            sqlx::query("SELECT 1").execute(m.db()).await
                .map(|_| "ready")
                .map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE)
        }))
}
```

### Fix 5 — Commit Cargo.lock

Remove `Cargo.lock` from `.gitignore`, then `git add Cargo.lock`.

### Fix 6 — Configurable pool

```rust
// model/store/mod.rs
let max_conns = std::env::var("SERVICE_DB_MAX_CONNS")
    .unwrap_or_else(|_| "5".to_string())
    .parse::<u32>().unwrap_or(5);
let acquire_timeout = std::env::var("SERVICE_DB_ACQUIRE_TIMEOUT_MS")
    .unwrap_or_else(|_| "5000".to_string())
    .parse::<u64>().unwrap_or(5000);
PgPoolOptions::new()
    .max_connections(max_conns)
    .acquire_timeout(Duration::from_millis(acquire_timeout))
    .connect(&config().DB_URL).await
```

### Fix 7 — Observability

Replace `tracing_subscriber::fmt().without_time().with_target(false)` with `.json()` + keep target + timestamps.

---

## 9. Production Readiness Table (26 items)

| # | Item | Severity | Status |
|---|---|---|---|
| 1 | Container panics on startup | Critical | ❌ |
| 2 | Secrets committed to git | Critical | ❌ |
| 3 | Container runs as root | Critical | ❌ |
| 4 | `init_dev` destructive `DROP DATABASE` | Critical | ❌ |
| 5 | No graceful shutdown | Critical | ❌ |
| 6 | No health/readiness endpoint | High | ❌ |
| 7 | Pool fixed 5, no acquire timeout | High | ❌ |
| 8 | No DB migrations | High | ❌ |
| 9 | `Cargo.lock` gitignored | High | ❌ |
| 10 | No CORS | High | ❌ |
| 11 | No rate limiting | High | ❌ |
| 12 | Debug logging in prod | High | ❌ |
| 13 | No TLS | High | ❌ |
| 14 | Logs lack timestamps/structured fields | High | ❌ |
| 15 | No metrics/Prometheus | Medium | ❌ |
| 16 | Image ~1.2 GB | Medium | ❌ |
| 17 | No build caching in Dockerfile | Medium | ❌ |
| 18 | No restart policy | Medium | ❌ |
| 19 | No DB volume | Medium | ❌ |
| 20 | `tokio` `full` features excessive | Medium | ⚠️ |
| 21 | `unwrap` on startup | Low | ❌ |
| 22 | SQL password typo | Low | ❌ |
| 23 | Compose `version: "3.8"` deprecated | Low | ❌ |
| 24 | No Dependabot | Low | ❌ |
| 25 | `web-folder/` missing | Low | ❌ |
| 26 | `actions/checkout@v3` outdated | Low | ❌ |
