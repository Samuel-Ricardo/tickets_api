# ADR-002: SQLx + sqlb Runtime Builder over `query!` Macros

- **Status:** Accepted (with risk)
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [data-models.md](../data-models.md), [security-review.md](../security-review.md) S7/S14/S15

## Context

The data layer (`base/db.rs`) needed generic CRUD over PostgreSQL driven by a marker trait (`DbBmc`) so that `UserService` and `TaskService` share one implementation. Two strategies were available: compile-time-checked `sqlx::query!`/`query_as!` macros, or a runtime query builder.

## Decision

Use **SQLx 0.7** (runtime-tokio-rustls, postgres, uuid, time features) with **sqlb 0.4** building SQL at **runtime**: `sqlb::raw_builder` select/insert/update/delete with `and_where`, `returning("id")`, `not_none_fields`.

The `macros` feature on SQLx is enabled but **unused** — no `query!`/`query_as!` anywhere in the codebase.

## Consequences

**Positive**
- One generic implementation serves all CRUD shapes (`base/db.rs`) — no repetitive SQL strings
- `NOT NULL`-unaware inserts via `not_none_fields` are ergonomic for optional update fields
- Parameter binding is used consistently — no string-concatenated SQL → **no SQL injection** (OWASP A03 passes in app code)

**Negative**
- **Schema drift surfaces at runtime, not compile time.** The current schema `user.username / task.name / tasks(plural)` versus models `User.name / Task.title / TABLE="tasks"` compiles fine and fails only at query time (P0-5, S15)
- Runtime discovery makes migrations and tests more error-prone
- `sqlb 0.4` is a small ecosystem-maintained crate — verify maintenance cadence

## Alternatives considered

- **`sqlx::query!` macros** — compile-time SQL validation, but generics across varied model shapes become painful; the `DbBmc` generic style would need async-trait gymnastics or per-entity macro calls
- **Diesel** — heavier ORM, compile-time DSL, doesn't fit the dynamic-`Fields` style used here
- **SeaORM** — full ORM, more abstraction than the project needs

## Related work

- P0-5/P1-6: fix schema drift and adopt `sqlx::migrate!` for compile-time-checked schema
- Keep `sqlb`'s parameterization; never regress into raw string SQL