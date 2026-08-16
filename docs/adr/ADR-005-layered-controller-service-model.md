# ADR-005: Layered Controller–Service–Model with Axum `State` DI

- **Status:** Accepted (with risk: 3 layering inversions)
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [architecture.md](../architecture.md) § 2–3

## Context

The project needed a structure that stays maintainable as features grow: HTTP concerns separate from business logic separate from data access. Mechanical DI containers were considered overkill for a crate this size, so the team chose explicit layering with axum's `State` to thread dependencies.

## Decision

Layer the crate:

```
router/ → controller/ + rpc/ → service/ → base/db.rs → model/store
```

- `controller/` = HTTP handlers; `rpc/` = JSON-RPC dispatcher + method wrappers (the active path)
- `service/` = thin domain operations (`UserService`, `TaskService`) implementing `DbBmc` for table binding
- `base/db.rs` = generic CRUD (`get/list/create/update/delete`) over sqlb
- `model/` = `ModelManager` (pool holder) + entity structs + `store` (pool factory)
- DI via clone-able `ModelManager` injected with `State`; no DI container

## Consequences

**Positive**
- Clear mental model; generic `base::db` removes CRUD duplication
- `State`-based DI is type-safe and testable (swap pool for a mock)
- Error-flow intent exists (see ADR-006) even where execution is broken today

**Negative**
- **3 inverted dependency edges** (infra depending on presentation/domain):
  - `middleware/auth → controller` (imports `set_token_cookies`, `TicketController`)
  - `model → service` (unused `use crate::service;`)
  - `base/db → model` (infra depends upward on domain)
- **Broken `From` bridge:** `From<model::error::Error>` commented out → forces `.unwrap()` at the service/controller boundary (breaks the whole flight of errors)
- Two parallel presentation styles coexist: active RPC + dormant REST stub (Q2/Q3)

## Alternatives considered

- **Full hexagonal/ports-and-adapters** — overkill for this scope; the layered chain captures the intent more simply
- **DI container (e.g., `shaku`/`injectables`)** — ceremony without payoff at this size
- **Single-file monolith** — rejected; the project already benefits from separation

## Related work

- P0-6: uncomment/fix `From` bridges, purge `.unwrap()`s
- P2-5: pick one presentation style (RPC vs REST); delete the dormant stack
- Fix dependency direction: extract cookie/session logic out of `controller` into a neutral module (`auth::session`) so middleware no longer imports presentation code