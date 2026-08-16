# ADR-004: `OnceLock<Config>` Env-Only Singleton

- **Status:** Accepted (with risk)
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [architecture.md](../architecture.md) § 6, [deployment-operations.md](../deployment-operations.md) § 4

## Context

The app needs process-wide configuration (DB URL, keys, durations, static folder) shared across modules with zero ceremony. Options: a config crate, manual `Arc<RwLock<Config>>` injection through axum `State`, or a lazy global singleton. The project chose the singleton to keep handler signatures clean (config is read anywhere, including inside `base::db`/`crypt`, without threading it through every call).

## Decision

Use `std::sync::OnceLock<Config>`:

```rust
static CFG: OnceLock<Config> = OnceLock::new();
pub fn config() -> &'static Config { CFG.get_or_init(|| load_from_env().expect("FATAL - WHILE LOADING CONFIG")) }
```

`load_from_env` reads 6 `SERVICE_*` env vars; keys are base64url-decoded to `Vec<u8>`; duration parsed as `f64` seconds.

## Consequences

**Positive**
- One-liner access anywhere; stateless and dead-simple
- Env-driven (12-factor III in spirit); container-compatible when env is supplied
- `OnceLock` is sync after init — zero contention on reads

**Negative**
- **Hard panic** on any missing/malformed var — the container now crashes rather than failing fast with a useful message (S10(d), Atlas D1/D2)
- Global mutable-ish design complicates testing (tests share one config; can't re-load with different values)
- Dev defaults in `.cargo/config.toml [env]` do **not** apply to the bare binary — deployment confusion (D2)
- Secrets live in env defaults committed to git (S2)

## Alternatives considered

- **Config through axum `State`** — cleanest DI, but every helper that needs config must receive it; high churn for `base::db`/`crypt`
- **`figment`/`config` crate** — more features (files, layering) than needed here
- **`LazyLock<T>`** — same shape, newer; `OnceLock` is fine

## Related work

- P0-7: provide env in compose + never panic-on-missing-env in prod (return `Result` from `config()`; log once + retry or exit gracefully)
- P2-9: `SERVICE_DB_MAX_CONNS` / `SERVICE_DB_ACQUIRE_TIMEOUT_MS` extensions