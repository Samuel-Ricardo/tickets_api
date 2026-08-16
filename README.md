# tickets_api

> A Rust web service for ticket/task management with cookie-based token auth, built on **Axum 0.7** + **SQLx 0.7** + **PostgreSQL**.

> ⚠️ **Project status: early development — NOT production ready.** The authentication subsystem is currently inoperative, Docker builds produce a crash-looping container, and several security blockers are documented in this README. See [Status & Known Blockers](#status--known-blockers) below for the full list.

---

## Table of contents

- [Overview](#overview)
- [Status & Known Blockers](#status--known-blockers)
- [Features](#features)
- [Tech stack](#tech-stack)
- [Repository layout](#repository-layout)
- [Quick start](#quick-start)
- [HTTP API at a glance](#http-api-at-a-glance)
- [Configuration](#configuration)
- [Development database](#development-database)
- [Testing](#testing)
- [Containerization & deployment](#containerization--deployment)
- [Security posture](#security-posture)
- [Documentation index](#documentation-index)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

`tickets_api` is a small REST-style API created as a learning/teaching project (it accompanies the *"Ultimate Rust Web Server"* course lineage). It exposes a JSON-RPC-style endpoint over HTTP for managing **tasks**, plus login/logoff flows backed by a **signed-cookie** token mechanism. The repository currently mixes two presentation styles:

1. **Active** — `POST /api/rpc` dispatching `create_task` / `list_tasks` / `update_task` / `delete_task` against a PostgreSQL database via `sqlb` builders.
2. **Dormant** — a classic per-resource REST stack (`/api/ticket`, `/api/tickets`) backed by an **in-memory** `TicketController` (`Arc<Mutex<…>>`). These routes exist in source but are **commented out** in `server.rs` and not mounted.

The intended architecture is a clean layered design — `router → controller/rpc → service → base/db → store` — with an axum `State`-injected `ModelManager` (a `sqlx` connection pool). The project also demonstrates a custom error → HTTP mapping pipeline via response middleware.

> **Important caveat:** As of the latest `v2` branch state, the application cannot currently run end-to-end. The blockers are detailed below and are the subject of the accompanying analysis documents.

## Status & Known Blockers

A multi-dimensional review (architecture, security/quality, deployment/operations) was performed against the latest code. The consolidated verdicts:

| Dimension | Verdict | Source |
|---|---|---|
| Architecture | Solid layered design with 6 P0 defects and 3 layering inversions | [docs/architecture.md](docs/architecture.md) |
| Security / Quality | ❌ **REJECT** — auth inoperative, secrets in git, panic-based errors | [docs/security-review.md](docs/security-review.md) |
| Deployment / Ops | ❌ **0/10 readiness** — container panic-crashes at startup | [docs/deployment-operations.md](docs/deployment-operations.md) |

### Six P0 blockers (the app cannot run end-to-end)

| # | Blocker | Location |
|---|---------|----------|
| 1 | **CTX is never inserted into request extensions** → every `POST /api/rpc` (auth-gated) returns 403 | `middleware/auth.rs:38-44` |
| 2 | **Token generation is `todo!()`** → `POST /api/login` panics | `crypt/token.rs:17-19` |
| 3 | **Dev DB bootstrap runs on every startup** and would `DROP DATABASE` in production | `server.rs:16`, `_dev_utils/dev_db.rs:14-15` |
| 4 | **Schema ⇔ code drift** (`username` vs `name`, `task` vs `tasks`, `title` vs `name`) | `sql/dev_initial/01-create-schema.sql` vs `service/user.rs`, `service/task.rs` |
| 5 | **`unwrap()`/panic on the request path** — any DB error or bad RPC params panics the handler | `rpc/task/mod.rs`, `rpc/mod.rs` |
| 6 | **Cookie missing `Secure`/`SameSite`** → CSRF surface | `controller/login.rs:21-23` |

Additional critical findings: hardcoded HMAC keys & DB credentials committed to git, password "hashing" uses keyed HMAC-SHA512 (not a KDF such as argon2id), Docker runs as root with a 1.2 GB toolchain image, `Cargo.lock` is gitignored (non-reproducible builds), and the existing CI pipeline has no `fmt`/`clippy`/`test`/`audit` gates.

The full, prioritized remediation roadmap lives in [docs/issue-roadmap.md](docs/issue-roadmap.md).

## Features

- **Task CRUD** via a JSON-RPC dispatcher (`POST /api/rpc`)
- **Login / logoff** with signed-cookie token auth (`auth-token` cookie)
- **Custom token scheme** — `b64u(ident).b64u(exp).sign` HMAC-SHA512 signature with per-user `token_salt`
- **Per-user UUID salts** for password and token (intended per-user isolation)
- **Generic CRUD plumbing** (`base::db`) parameterized by a `DbBmc` marker trait over `sqlb` builders
- **Error → HTTP mapping** via a single response-middleware point with per-request `req_uuid`
- **Config singleton** — `OnceLock<Config>` loaded from environment
- **Tracing** via `tracing`/`tracing-subscriber` with `EnvFilter`

## Tech stack

| Purpose | Crates |
|---|---|
| Web / HTTP | `axum 0.7.4`, `tower-http 0.5` (`fs`), `tower-cookies 0.10` |
| Async runtime | `tokio 1` (`full`), `async-trait 0.1` |
| Database | `sqlx 0.7` (`postgres`, `macros`, `runtime-tokio-rustls`, `uuid`, `time`), `sqlb 0.4` (runtime SQL builder) |
| Crypto | `rand 0.8`, `hmac 0.12`, `sha2 0.10`, `base64-url 2.0` |
| Serialization | `serde 1`, `serde_json 1`, `serde_with 3` |
| Observability | `tracing 0.1`, `tracing-subscriber 0.3` (`env-filter`) |
| Utility | `strum_macros 0.26`, `uuid 1` (`v4`, `fast-rng`, `serde`), `lazy-regex 3`, `time 0.3` |
| Dev / test | `anyhow 1`, `httpc-test 0.1`, `serial_test 3` |

Rust **edition 2021**; the Dockerfile pins `rust:1.76`; the container exposes **8080**; PostgreSQL runs on host port **5433** in the dev compose stack.

## Repository layout

```
tickets_api/                      # ← repository root (git, MIT LICENSE, this README)
├── .github/workflows/
│   └── docker-publish.yml        # Buildx + push to ghcr.io + cosign sign (no quality gates)
├── .gitignore                    # NOTE: Cargo.lock is gitignored (should be committed for a binary)
├── LICENSE                       # MIT — Copyright (c) 2024 Samuel_Ricardo
├── README.md                     # this file
└── tickets_api/                  # the Cargo workspace member (the actual app)
    ├── .cargo/config.toml        # [env] for `cargo` subcommands only (NOT the bare binary)
    ├── .dockerignore             # single line: **/target
    ├── Cargo.toml                # deps + dev-deps + commented cargo-watch recipes
    ├── Dockerfile                # two-stage, both rust:1.76 (runtime = full toolchain)
    ├── docker-compose.yaml       # api + db_test (postgres), dev only
    ├── sql/dev_initial/          # 00-recreate-db.sql, 01-create-schema.sql, 02-dev-seed.sql
    └── src/
        ├── main.rs               # entry point; spawns server::startup()
        ├── server.rs             # bootstrap: routes, layers, bind 0.0.0.0:8080
        ├── ctx.rs                # CTX { user_id }
        ├── config/               # OnceLock<Config> env loader
        ├── base/db.rs            # generic DbBmc CRUD over sqlb
        ├── controller/           # hello, login/logoff, legacy in-memory ticket controller
        ├── crypt/                # pwd (HMAC-SHA512), token (signed), errors
        ├── error/               # 21-variant Error enum → HTTP mapping
        ├── log/                 # request log line
        ├── middleware/           # auth (ctx resolver, require_auth), response mapper
        ├── model/               # ModelManager, user/task/ticket/login models, store (pool)
        ├── observability/        # tracing-subscriber init
        ├── router/             # hello, login, (dormant) tickets routes + statics
        ├── rpc/                 # JSON-RPC dispatcher + task methods
        ├── service/            # UserService, TaskService (thin CRUD wrappers)
        ├── util/                # base64, time helpers
        └── _dev_utils/          # dev DB bootstrap (runs at every startup today)
```

The detailed, type-level module map is in [docs/module-map.md](docs/module-map.md).

## Quick start

> Requires a running PostgreSQL instance reachable at the configured `SERVICE_DB_URL` and the required environment variables below.

### Local (cargo)

The `.cargo/config.toml` `[env]` block applies environment variables **to `cargo` subcommands only** (`cargo run`, `cargo test`). It sets sensible dev defaults so `cargo run` works locally against a local Postgres on `localhost:5432`.

```bash
cd tickets_api
cargo run
# server binds 0.0.0.0:8080
```

### Docker Compose (dev)

```bash
cd tickets_api
docker compose up --build
# api → 0.0.0.0:8080   ;   db_test (postgres) → host 5433
```

> ⚠️ The compose `api` service **only sets `RUST_LOG` and `SERVICE_WEB_FOLDER`** and is missing `SERVICE_DB_URL`, `SERVICE_PWD_KEY`, `SERVICE_TOKEN_KEY`, and `SERVICE_TOKEN_DURATION_SEC`. Because `.cargo/config.toml [env]` does **not** apply to the bare `./tickets_api` binary launched by the Dockerfile `CMD`, the container **panics at startup** loading config, and `_dev_utils::init_dev()` panics even earlier on the unresolvable `db` hostname. See [docs/deployment-operations.md](docs/deployment-operations.md) for the fixed compose file and required code changes.

## HTTP API at a glance

| Method | Path | Auth | Body in | Body out |
|---|---|---|---|---|
| GET | `/` | No | — | `{"hello":"world"}` |
| POST | `/api/login` | No | `Json<LoginPayload>` `{ username, pwd }` | `{"result":{"success":true}}` + sets `auth-token` cookie |
| POST | `/api/logoff` | No | `Json<LogoffPayload>` `{ logff }` | `{"result":{"logged off": bool}}` + removes cookie |
| POST | `/api/rpc` | **Yes** | `Json<RpcRequest>` `{ id?, method, params? }` | `{"id": …, "result": …}` |
| * | `/*` | No | — | static files from `web-folder/` fallback |

RPC methods dispatched at `POST /api/rpc`:

| `method` | `params` | `result` |
|---|---|---|
| `create_task` | `{"data":{"title": String}}` | `{"id": i64, "title": String}` |
| `list_tasks` | — | `[{"id": i64, "title": String}]` |
| `update_task` | `{"id": i64, "data":{"title": String?}}` | `{"id": i64, "title": String}` |
| `delete_task` | `{"id": i64}` | `{"id", "title"}` |
| *(anything else)* | — | **panics** (`Error::UnkownRpcMethod`) — no JSON-RPC error envelope |

> The dormant REST ticket endpoints (`POST /api/ticket`, `DELETE /api/ticket/:id`, `GET /api/tickets`) are defined in `router/tickets.rs` but commented out in `server.rs`.

Full request/response examples and the error envelope format are in [docs/api-reference.md](docs/api-reference.md).

## Configuration

All configuration is read from environment variables (12-factor style) by `src/config/mod.rs`. `config()` returns a lazily-initialized `&'static Config` (an `OnceLock`); a missing/malformed variable causes a **hard panic** (`"FATAL - WHILE LOADING CONFIG"`).

| Variable | Purpose | Dev default (`.cargo/config.toml`) |
|---|---|---|
| `SERVICE_DB_URL` | PostgreSQL connection string | `postgresql://app_user:dev_only_pwd@localhost:5432/app_db` |
| `SERVICE_WEB_FOLDER` | Static-file fallback directory | `web-folder/` |
| `SERVICE_PWD_KEY` | HMAC-SHA512 key for password hashing (base64url) | `ZGV2X29ubHlfcHdkX2tleQ==` (decodes to ASCII `dev_only_pwd_key`) |
| `SERVICE_TOKEN_KEY` | HMAC-SHA512 key for token signing (base64url) | `ZGV2X29ubHlfdG9rZW5fa2V5` (decodes to ASCII `dev_only_token_key`) |
| `SERVICE_TOKEN_DURATION_SEC` | Token lifetime (seconds, `f64`) | `1800` (30 min) |
| `RUST_LOG` | `tracing` env filter | `tickets_api=debug` |

> 🔴 **Security warning — do not reuse these values.** The dev `SERVICE_PWD_KEY` and `SERVICE_TOKEN_KEY` are publicly committed, ~17/18-byte ASCII strings with near-zero entropy. Anyone with repo access can forge tokens and offline-brute-force the password database. See [docs/security-review.md](docs/security-review.md) (finding S2) for rotation and remediation guidance. Also note `00-recreate-db.sql` creates `app_user` with password **`dev_only_pws`** (a typo) — inconsistent with the `dev_only_pwd` used by `config.toml` and `docker-compose.yaml`.

## Development database

The dev SQL bootstrap lives in `sql/dev_initial/`:

- `00-recreate-db.sql` — terminates backends, `DROP DATABASE app_db`, `DROP USER app_user`, recreates user/db (with the password typo mentioned above).
- `01-create-schema.sql` — creates `user` (`id`, `username`) and `task` (`id`, `name`, `pwd`, `pwd_salt`, `token_salt`). Note: salts live on `task` today, and the schema diverges from the Rust models (see [docs/data-models.md](docs/data-models.md)).
- `02-dev-seed.sql` — inserts user `demo1`.

`_dev_utils::init_dev()` runs this bootstrap on **every** server startup today (it is not feature-gated). It hardcodes `postgres://postgres:welcome@db:5432/postgres` and swallows errors with `let _ =`. This is destructive and must not run in production — see finding S7 in [docs/security-review.md](docs/security-review.md).

## Testing

Tests live inline in `src/` under `#[cfg(test)]`. **Effective coverage is ~15%** of security-critical code, and several test functions are missing the `#[tokio::test]`/`#[test]` attribute and never run:

- ✅ Running: `src/service/task.rs` — `test_create`, `test_list`, `test_get_err_not_found`, `test_delete_err_not_found` (all `#[serial] #[tokio::test]`, need a live Postgres + `init_dev()` seeding).
- ❌ Never run: `service/user.rs::test_first_ok_demo1`, `service/task.rs::test_update`, `crypt/mod.rs::test_encrypt_into_b64url`, `crypt/token.rs::test_generate_web_token` (missing test attributes).
- ❌ The `httpc-test` dev-dep (an HTTP fixture client) is declared but **unused** — there are no HTTP-layer tests.

```bash
cargo test --all-targets   # requires a running, seeded Postgres for the serial DB tests
```

See the test-coverage section of [docs/security-review.md](docs/security-review.md) for the gap matrix.

## Containerization & deployment

The repository contains a working Docker build path and a CI image-publishing workflow, but the resulting images are **not deployable** as-is. Highlights:

- **`Dockerfile`** — two stages, both `rust:1.76` (the runtime stage ships the entire ~1.2 GB toolchain); runs as **root**; no `USER`, no `HEALTHCHECK`, no `--locked`; `COPY . .` with a poor `.dockerignore` (`**/target` only) bakes `.git/`, `.cargo/` secrets, and `sql/` into the context; no dependency-cache layer ordering.
- **`docker-compose.yaml`** — `version: "3.8"` (deprecated); `api` service missing 4 of 6 required env vars → panic at startup; no healthchecks, no `depends_on` condition, no DB volume, no network, no restart policy.
- **CI (`.github/workflows/docker-publish.yml`)** — good supply-chain hygiene (pinned action SHAs, GHA build cache, skip-push on PR, keyless **cosign** signing to ghcr.io) but **zero quality gates** before the image build (no `fmt`/`clippy`/`test`/`cargo audit`); `actions/checkout@v3` is outdated; no Dependabot.

A hardened multi-stage Dockerfile, a fixed `docker-compose.yaml`, a `.env` secrets template, a complete `ci.yml` quality workflow, and the source-code changes required for the container to boot are provided in [docs/deployment-operations.md](docs/deployment-operations.md).

## Security posture

The full 15-finding (S1–S15) + 10 quality-finding (Q1–Q10) review, the OWASP Top 10 checklist, and the prioritized remediation roadmap live in [docs/security-review.md](docs/security-review.md). Headline:

- 🔴 **S1 CRITICAL — auth inoperative**: token generation is `todo!()` (login panics) *and* `mw_ctx_resolver` never inserts `CTX` into request extensions (every authenticated request fails). [⇢ fix sketch in the doc]
- 🔴 **S2 CRITICAL — keys & DB creds in git** (the ASCII dev keys, `dev_only_pwd`, `postgres/welcome`).
- 🟠 **S3 HIGH — password storage is keyed HMAC-SHA512**, not a KDF; non-constant-time compare; offline crack >1M guesses/sec. Recommends argon2id.
- 🟠 **S4 HIGH — remote DoS** via panic/`unwrap` in the RPC path (35 unwrap/expect/todo sites).
- 🟠 **S7 HIGH — `init_dev()` `DROP DATABASE` runs on every startup** in the production code path.
- 🟠 **S8 HIGH — broken access control**: `ctx` is ignored by `base::db` — no ownership scoping (IDOR).

OWASP snapshot: A03 Injection **passes** (sqlb is parameterized); A01, A02, A04, A05, A07, A08 **fail**; A09 Logging **partial**. See the doc for the full checklist.

## Documentation index

All detailed analysis lives in the [`docs/`](docs/) directory:

| Document | Description |
|---|---|
| [docs/README.md](docs/README.md) | Documentation index & how to navigate |
| [docs/overview.md](docs/overview.md) | Project overview, goals, status, branches, history |
| [docs/architecture.md](docs/architecture.md) | Architecture style, module dependency graph, layering, DI, error/auth/config arch, issues |
| [docs/api-reference.md](docs/api-reference.md) | HTTP endpoints + RPC methods + examples + error envelope |
| [docs/module-map.md](docs/module-map.md) | Type-level module-by-module map |
| [docs/data-models.md](docs/data-models.md) | Rust models ↔ DB schema & the drift |
| [docs/auth-flow.md](docs/auth-flow.md) | Token format, cookie lifecycle, sequence diagram, weaknesses |
| [docs/security-review.md](docs/security-review.md) | Security (S1–S15), quality (Q1–Q10), tests, OWASP, roadmap |
| [docs/deployment-operations.md](docs/deployment-operations.md) | Docker, compose, env, observability, CI/CD, hardened artifacts |
| [docs/issue-roadmap.md](docs/issue-roadmap.md) | Consolidated issue register + remediation sequencing |
| [docs/adr/README.md](docs/adr/README.md) | Architecture Decision Records index (ADR-001…006) |

## Contributing

This project is a learning codebase. To work on it safely:

1. Read the [Known Blockers](#status--known-blockers) above and the [issue roadmap](docs/issue-roadmap.md).
2. Start with the P0 fixes (auth `todo!()` + `CTX` insertion, feature-gating `_dev_utils`, schema alignment, removing `unwrap` from request paths).
3. Run `cargo fmt`, `cargo clippy -- -D warnings` (recommended: `-W clippy::unwrap_used`), and `cargo test` before submitting.
4. Commit `Cargo.lock` (remove it from `.gitignore` — it is a binary crate).
5. Never commit secrets; externalize them via `SERVICE_*` env vars / a gitignored `.env`.

## License

[MIT License](LICENSE) — Copyright (c) 2024 Samuel_Ricardo. See [`LICENSE`](LICENSE) for the full text.
