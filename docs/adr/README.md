# Architecture Decision Records (ADRs)

This directory records the architectural choices made (or recommended) for `tickets_api`, along with their context, consequences, and status.

> ADRs here are derived from the codebase review. Where a decision is *implemented as-is* the status is **Accepted**; where the analysis recommends a change, the ADR documents the current choice and links to the remediation finding.

## Index

| ADR | Decision | Status |
|---|---|---|
| [ADR-001](ADR-001-axum-web-framework.md) | Use **Axum** + Tower middleware as the web framework | Accepted |
| [ADR-002](ADR-002-sqlx-sqlb-runtime-builder.md) | Build SQL with **`sqlb`** at runtime rather than `sqlx::query!` macros | Accepted (with risk: S15 schema drift) |
| [ADR-003](ADR-003-signed-cookie-token-auth.md) | **Custom signed-cookie token** auth instead of JWT bearer | Accepted (blocked by S1 `todo!()` / S6 cookie attrs) |
| [ADR-004](ADR-004-oncelock-config-singleton.md) | `OnceLock<Config>` **env-only singleton** loaded once | Accepted (with risk: hard `panic!` on missing var) |
| [ADR-005](ADR-005-layered-controller-service-model.md) | **Layered controller-service-model** with axum `State` DI | Accepted (with risk: 3 layering inversions) |
| [ADR-006](ADR-006-error-response-middleware-mapping.md) | **Error → HTTP via response middleware** with `req_uuid` | Accepted (with risk: default arm = 500) |

## Format

Each ADR follows a lightweight Michael-Nygard-style template:

- **Title** · **Status** · **Date**
- **Context** — the forces at play
- **Decision** — the choice made
- **Consequences** — positive, negative, and neutral
- **Alternatives considered**
- **Related findings** — links into [../security-review.md](../security-review.md) and [../issue-roadmap.md](../issue-roadmap.md)

## How to add an ADR

1. Copy `ADR-NNN-template` numbering from the next free number.
2. Use `kebab-case` file names: `ADR-NNN-short-slug.md`.
3. Add a row to the table above and to [../issue-roadmap.md](../issue-roadmap.md) if the ADR surfaces a new tracked issue.
