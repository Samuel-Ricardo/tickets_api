# ADR-006: Error → HTTP via Response Middleware

- **Status:** Accepted (with risk: default arm = 500)
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [architecture.md](../architecture.md) § 4, [api-reference.md](../api-reference.md) § Status mapping, [security-review.md](../security-review.md) S9

## Context

Handlers return `Result<T, Error>`; the HTTP layer needs to translate application errors into consistent client-facing JSON without leaking internals, in one place, rather than scattering `status()` calls per handler. The classic axum approach is `impl IntoResponse for Error` — but a single centralized mapper keeps formatting identical across all routes.

## Decision

Two-stage pipeline:

1. `impl IntoResponse for Error` → always `500` + `self` inserted into `res.extensions_mut()` (so the error survives the response pipeline)
2. `main_response_mapper` (`map_response` layer) extracts the `Error`, computes `client_status_and_error()` → `(StatusCode, ClientError)` with `req_uuid = Uuid::new_v4()`, builds body `{"error":{"type": client_error.as_ref(), "req_uuid": uuid}}`, and calls `log_request` at `debug!`

## Consequences

**Positive**
- Single mapping point — consistent client body everywhere; no internal `Error` detail in *responses* (only `type` + `req_uuid`)
- `req_uuid` correlation for logs and support
- `IntoResponse` extension-insert keeps the mapper middleware-agnostic

**Negative**
- **Default arm `_ ⇒ 500 SERVICE_ERROR`**: unmapped variants (`TokenExpired`, `ValidationFail`, `UserNotFound`, crypt/time failures) all surface as 500 — wrong HTTP semantics, ops blindness (S9)
- `log_request` serializes the full internal `Error` (incl. `user_id`, `CtxCreationFail(String)`) at `debug!` — with prod at `RUST_LOG=debug`, internals leak to logs; no auth-failure/audit events
- Mapper is a `map_response` layer — errors generated *after* the layer (e.g., in fallback `ServeDir`) bypass it

## Alternatives considered

- **Per-handler `IntoResponse`** — scattering formatting; inconsistent bodies
- **`Error::client_status_and_error` directly in handlers** — couples handlers to mapping logic
- **Two middleware layers (error-capture + response-mapper)** — more moving parts for same result today

## Related work

- P2-1: explicit arms (401 token/403 creds/404 not-found/400 validation); unified `INVALID_CREDENTIALS`; strip internal `Debug` from request logs
- P1-1…P1-6 bounds: the mapper stays the single entry for client errors — including the JSON-RPC error envelope (S4) that should parallel it for `rpc_handler`