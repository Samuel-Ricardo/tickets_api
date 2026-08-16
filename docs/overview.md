# Overview

`tickets_api` is a small Rust web service for managing tickets/tasks. It was built as a learning project accompanying the *"Ultimate Rust Web Server"* course lineage and demonstrates an axum-based API with a custom signed-cookie token auth scheme, a layered service/repository design, and an error→HTTP mapping pipeline.

> Repository: `github.com:Samuel-Ricardo/tickets_api.git` · License: MIT (Copyright (c) 2024 Samuel_Ricardo)

## Goals (inferred)

- Demonstrate a clean **layered architecture** (router → controller/rpc → service → base/db → store) on axum.
- Implement **task CRUD** over PostgreSQL using a runtime SQL builder (`sqlb`) rather than compile-time `sqlx::query!` macros.
- Showcase **cookie-based signed-token auth** with per-user UUID salts as an alternative to JWT bearer tokens.
- Illustrate a single **error → HTTP response mapping** point with per-request correlation IDs (`req_uuid`).

## What it can do today (latest `v2`)

- Build and run locally via `cargo run` (where `.cargo/config.toml [env]` supplies dev defaults).
- Expose `POST /api/rpc` to dispatch `create_task` / `list_tasks` / `update_task` / `delete_task` against the dev database (once the DB is seeded and the schema matches the models).
- Expose `POST /api/login` and `POST /api/logoff`.
- Serve static files from `web-folder/` as a fallback.

## What it cannot do today (blockers)

The latest `v2` state cannot run end-to-end. See [../README.md § Status & Known Blockers](../README.md#status--known-blockers) for the six P0 blockers. In short: login panics (`todo!()` token generator), every authenticated request fails (CTX never inserted into extensions), the dev DB bootstrap runs destructively on every startup, the SQL schema diverges from the Rust models, RPC errors panic instead of returning a JSON-RPC error, and the cookie lacks `Secure`/`SameSite`.

## Branches

The repository keeps (at least) three long-lived branches:

| Branch | Role |
|---|---|
| `main` | primary line |
| `v1` | an earlier iteration |
| `v2` | the line analyzed here (HEAD at review time) |

History is emoji-prefixed conventional commits (e.g. `[ :passport_control: ] setup: auth (app::server)`, `[ :sparkles: ] setup: rpc - routes > task (app::server)`), keeping the *Ultimate Rust Web Server* course module naming (`app::server`, `app::model`, …).

## Status summary

| Dimension | Verdict | Detail |
|---|---|---|
| Architecture | Solid skeleton, 6 P0 defects, 3 layering inversions | [architecture.md](architecture.md) |
| Security / Quality | ❌ REJECT | [security-review.md](security-review.md) |
| Deployment / Ops | ❌ 0/10 readiness | [deployment-operations.md](deployment-operations.md) |
| Test coverage | ~15% of security-critical paths; a no-op test suite | [security-review.md § Test coverage](security-review.md) |
| Reproducibility | Broken — `Cargo.lock` is gitignored | [deployment-operations.md](deployment-operations.md) |

## Where to go next

- New to the code? → [module-map.md](module-map.md) then [architecture.md](architecture.md).
- Updating the API contract? → [api-reference.md](api-reference.md) and [data-models.md](data-models.md).
- Fixing auth? → [auth-flow.md](auth-flow.md) and [security-review.md](security-review.md) (S1).
- Shipping/deploying? → [deployment-operations.md](deployment-operations.md).
- Planning work? → [issue-roadmap.md](issue-roadmap.md).
