# ADR-001: Axum as the Web Framework

- **Status:** Accepted
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [architecture.md](../architecture.md), ADR-005, ADR-006

## Context

The project needed an async Rust web framework with: ergonomic handler extraction, Tower-compatible middleware, flexible routing/nesting, and a strong ecosystem for cookies and static files. Alternatives were Actix-web, Rocket, and axum. The project had already grown a JSON-RPC endpoint, cookie auth, and a static-file fallback before this review.

## Decision

Use **Axum 0.7.4** as the web framework, with `tower-http 0.5` (initially `fs` feature only), `tower-cookies 0.10`, and `tokio 1` as the async runtime.

Concretely: `Router` with `.merge()`/`.nest("/api", …)` composition, `State` injection, `FromRequestParts` extractors (`CTX`, `Cookies`), `route_layer(mw_require_auth)`, `map_response(main_response_mapper)`, `from_fn_with_state(mw_ctx_resolver)`, `CookieManagerLayer`, `fallback_service(ServeDir)`.

## Consequences

**Positive**
- Type-safe extractors and state plumbing; handler signatures are self-documenting
- Middleware composition is uniform Tower layers — easy to reason about order
- Route nesting cleanly separates `/api` (JSON) from `/` (static fallback)
- Ecosystem alignment: sqlx/time/uuid integrations all work with axum extracts

**Negative**
- Two dormant routes use **Axum 0.6 path syntax** (`/api/ticket/:id`) — 0.7 requires `{id}` (Q2)
- `FromRequestParts` for `CTX` depends on middleware inserting into extensions — a footgun the project tripped over (S1)

## Alternatives considered

- **Actix-web** — viable, but its extractor model differs; team already invested in axum idioms
- **Rocket** — heavy macro surface, less transparent middleware ordering
- **Handwritten hyper service** — too low-level for this project's needs

## Related work

- Upgrade path: axum 0.8 (P3-6) — mostly mechanical; `Path` syntax, `serve` API changes
- Add `tower-http` features: `cors`, `set-header`, `timeout`, `limit` (P3-2, P3-3)