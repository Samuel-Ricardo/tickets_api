# Security & Code-Quality Review

> **Reviewer:** Carla (QA/Security specialist) · **Date:** 2026-08-14
> **Method:** 8-dimension adversarial review + OWASP Top 10 mapping + code-security skill (HMAC/JWT/Docker/secrets rules applied)
> **Target:** `tickets_api` latest `v2` branch — 46 `.rs` files across 20 directories

## Headline verdict

> ❌ **REJECT — the authentication subsystem is inoperative (two independent blockers), secrets are committed to git, and error handling is panic-based. Do not deploy.**

The architecture is a solid skeleton, but the project is in an early/transitional development state. Fix the P0 items first, re-run the full matrix, then consider deployment.

---

## 1. Security Findings (S1–S15)

### S1 · CRITICAL — Authentication is completely non-functional

**Files:** `crypt/token.rs:17-19`, `middleware/auth.rs:38-44`, `middleware/auth.rs:72-81`

**Two independent blockers:**

1. `_generate_token()` is an empty `todo!()` — `generate_web_token()` therefore **panics** on every call. Login (`set_token_cookies` at `controller/login.rs:69`) and the middleware token-refresh path both hit it. **Every login request panics.**
2. `mw_ctx_resolver` computes `ctx_result` (auth.rs:38) but **never inserts it into `req.extensions`**. The `CTX` extractor reads `parts.extensions.get::<Result<CTX>>()`, always finds `None`, returns `AuthFailCtxNotInRequestExt`. **Every authenticated request fails.**

**Impact:** The API authenticates nobody. Login crashes (panic → connection abort); no `/api/rpc` call can ever succeed.

**Fix sketch:**
```rust
// token.rs — implement _generate_token
fn _generate_token(ident: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    let exp = now_utc_plus_sec_str(duration_sec);
    let sign_b64u = _token_sign_into_b64url(ident, &exp, salt, key)?;
    Ok(Token { ident: ident.to_string(), exp, sign_b64u })
}

// middleware/auth.rs — in mw_ctx_resolver, before next.run(req):
req.extensions_mut().insert(ctx_result);
```

Fix **both** before re-enabling any protected route, and add HTTP tests proving: no-cookie → 401/403, valid-cookie → 200, tampered-signature → rejected.

---

### S2 · CRITICAL — Hardcoded cryptographic keys and DB credentials committed to git

**Files:** `.cargo/config.toml:22-26`, `_dev_utils/dev_db.rs:14-15`, `sql/dev_initial/00-recreate-db.sql:14`, `docker-compose.yaml:30`

- `SERVICE_PWD_KEY = "ZGV2X29ubHlfcHdkX2tleQ=="` decodes to `dev_only_pwd_key` (17 bytes ASCII, ~zero entropy).
- `SERVICE_TOKEN_KEY = "ZGV2X29ubHlfdG9rZW5fa2V5"` decodes to `dev_only_token_key` (18 bytes).
- DB URL `postgresql://app_user:dev_only_pwd@localhost:5432/app_db` committed.
- `dev_db.rs` `postgres://postgres:welcome@db:5432/postgres` committed.
- Repo remote: `github.com:Samuel-Ricardo/tickets_api` (public-reachable).

**Impact:** Anyone with repo access can forge valid tokens and offline brute-force the entire password database (see S3).

**Recommendation:**
1. Rotate immediately if ever used beyond localhost.
2. Remove secrets from `.cargo/config.toml` (keep only non-secret env like `RUST_LOG`).
3. Externalize via env vars / secrets manager (`{env:...}`), inject in compose via `env_file` or `secrets`.
4. Purge from git history (`git filter-repo`).
5. Add `gitleaks`/`trufflehog` to CI.
6. Add `.gitignore` safeguards for `*.env`.

---

### S3 · HIGH — Password hashing is keyed HMAC-SHA512, not a password KDF

**Files:** `crypt/pwd.rs:5-20`, `crypt/mod.rs:15-28`, `crypt/token.rs:56-61`

- Stored form: `#01#{HMAC-SHA512(PWD_KEY, content ‖ salt)}` — single-pass, no work factor, no bcrypt/argon2.
- `validate_pwd` re-derives and compares strings — **non-constant-time**.
- Attacker with DB dump + key: `>1,000,000` guesses/sec/core vs `~100/sec` for bcrypt/argon2.

**Recommendation:** Replace with `argon2` crate + `password-hash`:
```rust
let hashed = Argon2::default().hash_password(pwd.as_bytes(), &salt)?;
Argon2::default().verify_password(pwd.as_bytes(), &hashed)?;
```
Keep `PWD_KEY` only as a pepper; add migration `#01#` → argon2 on next login.

---

### S4 · HIGH — Remote DoS via panic-based error handling

**Files:** `rpc/mod.rs:27-48,64`, `rpc/task/mod.rs:15,16,22,32,34,40,43`, `controller/ticket.rs:26,41,49`

- `exec_rpc_fn!` chains **7 `.unwrap()` calls** per RPC invocation.
- Any DB error, malformed JSON, or unknown method → panic → connection aborted.
- **35 `.unwrap()/.expect()/todo!()` sites** total in the repo (verified by grep).
- `TicketController` `Mutex::lock().unwrap()` panics on poisoning.

**Recommendation:** Propagate `Result`; map to JSON-RPC error `{"id":id,"error":{"code":-32602,"message":"..."}}`; replace `lock().unwrap()` with `map_err`; add `#![deny(clippy::unwrap_used)]` or CI gate `cargo clippy -D clippy::unwrap_used`.

---

### S5 · HIGH — No rate limiting/lockout; demo credential 'welcome'

**Files:** `router/login.rs:10`, `controller/login.rs:36-78`, `_dev_utils/dev_db.rs:22,58`

- `POST /api/login` has no throttling/lockout/CAPTCHA/backoff.
- Seed resets `demo1` password to `welcome` on every startup (`dev_db.rs:58`).

**Recommendation:** `tower_governor` or `axum-ratelimit` (5/min per-IP + per-username), lockout policy, remove the `welcome` reset in non-dev builds.

---

### S6 · HIGH — Session cookie missing Secure/SameSite; no server-side invalidation

**Files:** `controller/login.rs:16-27,29-34`

- Cookie: `HttpOnly` + `Path=/` only. **Missing:** `Secure`, `SameSite`, `Max-Age`/`Expires`, `Domain`.
- `remove_token_cookies` only deletes the client cookie; the token remains valid (no `token_salt` rotation).
- `mw_ctx_resolver` re-issues a fresh 30-min token on every request (**sliding expiry, no absolute cap**).

**Recommendation:** `set_secure(cfg.https_only)`, `set_same_site(SameSite::Strict)`, `set_max_age(30 min)`; add an absolute max-lifetime check in `_validate_token_sign_and_exp` (e.g., not older than 8 h); rotate `token_salt` in DB on logoff.

---

### S7 · HIGH — Dev DB resetter in production code path

**Files:** `main.rs:17-18`, `server.rs:16`, `_dev_utils/dev_db.rs:24-31,73-77`

- `_dev_utils` is `pub mod` **unconditionally** (`main.rs:18`); the comment says it was `#[cfg(test)]`-gated but it isn't.
- `startup()` calls `_dev_utils::init_dev()` (`server.rs:16`) in every build.
- `init_dev` connects to `postgres://postgres:welcome@db:5432/postgres` and executes `00-recreate-db.sql`: `pg_terminate_backend`, `DROP DATABASE`, `DROP USER`, `CREATE USER`, `CREATE DATABASE`.
- Result swallowed with `let _ =`.

**Recommendation:** `#[cfg(feature="dev-utils")] pub mod _dev_utils;` and gate `init_dev()` behind the same feature; build prod with `--no-default-features`.

---

### S8 · HIGH — Broken access control: no ownership checks

**Files:** `base/db.rs:16,40,72,98` (`_ctx` unused), `service/task.rs:19-38`, `controller/ticket.rs:40-54`

- `base::db` `list`/`get`/`update`/`delete` take `ctx: &CTX` and **ignore it** (`_ctx`).
- `TaskService::list` returns ALL tasks of ALL users; `delete`/`update` by id with no `cid`/owner predicate.
- `TicketController` same: `list_tickets(_ctx)` returns everything, `delete_ticket(_ctx, id)` deletes anything.
- In-memory Ticket ids derived from `store.len()` (`ticket.rs:28`).

**Impact:** IDOR — any authenticated user can read/delete every other user's data (OWASP A01).

**Recommendation:** Thread `ctx` into queries: add `cid` column to `task`, enforce `and_where("cid", "=", ctx.user_id() as i64)`.

---

### S9 · MEDIUM — Auth/validation failures → HTTP 500; error taxonomy leaks via logs

**Files:** `error/mod.rs:84-107`, `middleware/response.rs:22-43`, `log/mod.rs:28-46`

- `client_status_and_error()` maps only 3 auth variants + ticket-not-found; everything else (`LoginFailUsernameNotFound`, `LoginFailPwdNotMathing`, `TokenExpired`, `UserNotFound`, `ValidationFail`) → **500 SERVICE_ERROR**.
- Correct HTTP semantics absent (401 vs 403 vs 400 vs 404).
- `log_request` serializes full internal `Error` (`user_id`, `CtxCreationFail(String)`) into debug logs; with `RUST_LOG=debug` in prod, internal details go to logs.

**Recommendation:** Split client mapping into explicit arms (401 auth/token, 403 wrong-creds, 404 user-not-found, 400 validation); keep internal `Debug` out of request logs (log type + `req_uuid` only); unified `INVALID_CREDENTIALS` for anti-enumeration.

---

### S10 · MEDIUM — Docker/compose hardening gaps

**Files:** `Dockerfile:1-19`, `docker-compose.yaml:1-31`, `.dockerignore:1`

| Gap | Detail |
|---|---|
| (a) Final image is `rust:1.76` (~1.2 GB) | Full toolchain shipped as runtime base |
| (b) No `USER` directive | Runs as root |
| (c) Tags unpinned | No digest pinning (`rust:1.76`, `postgres`) |
| (d) Compose missing 4 of 6 env vars | `config::load_from_env` panics at startup |
| (e) No `HEALTHCHECK`/restart/`cap_drop` | DB port 5433 exposed with `dev_only_pwd` |
| (f) `.dockerignore` = `**/target` | Build context includes `.git`, `.cargo` secrets, SQL seeds |
| (g) `SERVICE_WEB_FOLDER=web-folder/` relative | Static server 404s (dir not in image) |

**Recommendation:** `debian:bookworm-slim` or distroless, `USER 10001`, pin digests, `HEALTHCHECK`, `env_file` + `read_only` + `cap_drop:[ALL]`, `.dockerignore` add `.git .cargo sql`. See [deployment-operations.md](deployment-operations.md) for hardened manifests.

---

### S11 · MEDIUM — Token scheme weaknesses

**Files:** `crypt/token.rs:21-36,38-49`, `crypt/mod.rs:18-27`, `middleware/auth.rs:47-66`

- (a) Sign comparison `new_sign_b64u != origin_token.sign_b64u` — **not constant-time**.
- (b) Custom 3-part scheme — no `alg`/`iss`/`aud`/`jti`/`nonce`; HMAC key is global (S2); `ident` is raw username (PII in cookie); `_token_sign_into_b64url` does `Ok(signature.unwrap())` — HMAC init failure panics.
- (c) `parse_token` regex is dead code — contradicts `FromStr`.

**Recommendation:** `subtle::ConstantTimeEq` on decoded bytes; use `user.id` (not username) in `ident` or encrypt the cookie; remove `unwrap()`s from `token.rs`; delete dead `parse_token`.

---

### S12 · MEDIUM — No security headers, no TLS, no CORS

**Files:** `server.rs:24-35`, `Cargo.toml:31`

- No `X-Content-Type-Options`, `X-Frame-Options`/CSP, `Referrer-Policy`, HSTS.
- Server binds plain HTTP `0.0.0.0:8080`; no CORS layer.

**Recommendation:** Enable `tower-http` `cors` + `set-header` features; explicit allowlist; `SecurityHeaders` layer; document TLS termination (reverse proxy); `SERVICE_HTTPS=true` → `Secure` cookie + HSTS.

---

### S13 · MEDIUM — Input validation gaps

**Files:** `model/login.rs:4-8`, `model/task.rs:12-19`, `rpc/model/mod.rs:3-8`

- `LoginPayload`/`TaskForCreate` strings unbounded at HTTP layer; DB `VARCHAR` limits surface as 500s.
- RPC `params = Option<Value>` — `from_value` accepts arbitrary extra fields (no `deny_unknown_fields`).
- No content-length/body-size cap.

**Recommendation:** `#[serde(deny_unknown_fields)]`; validate lengths at boundary (`username ≤ 128`, `pwd ≤ 256`, `title ≤ 256` → 400); `DefaultBodyLimit`/`RequestSizeLimitLayer`.

---

### S14 · LOW — Raw SQL execution naive semicolon split + password typo

**Files:** `_dev_utils/dev_db.rs:67-80`, `00-recreate-db.sql:14`

- `pexec` splits each `.sql` on every `;` (`content.split(';')`) — semicolons inside string literals/comments break scripts; FIXME documented.
- `00-recreate-db.sql` sets `app_user` password to `dev_only_pws` (typo) vs `dev_only_pwd` elsewhere.

**Recommendation:** Use `sqlx::migrate!` / `sqlx-cli` whole-file execution; fix the password typo.

---

### S15 · LOW/MEDIUM — Salt columns on wrong table

**Files:** `01-create-schema.sql:4-18`, `model/user.rs:22-37`, `model/task.rs:5-9`

- Schema puts `pwd`, `pwd_salt`, `token_salt` on the **`task`** table; `user` has only `id` + `username`.
- `UserForLogin`/`UserForAuth` expect those columns on `user`; field name `name` vs column `username`; `TaskService::TABLE = "tasks"` vs schema `task`; model `title` vs schema `name`.
- Every login query: `column "pwd_salt" of relation "user" does not exist`; `first_by_username` queries `name` not `username`.
- Per-user salt isolation (the good idea — `gen_random_uuid()` per row) is unreachable.

**Recommendation:** Align schema to models (see [data-models.md](data-models.md) § Reconciled schema); keep `gen_random_uuid()` defaults; adopt `sqlx::migrate`.

---

## 2. Code-Quality Findings (Q1–Q10)

| # | Finding | Affected files | Fix |
|---|---|---|---|
| Q1 | Dead code: `todo!()` + dead `parse_token` regex | `token.rs:17-19`, `auth.rs:84-96` | Implement/delete; `cargo clippy -D warnings -D clippy::todo` in CI |
| Q2 | Dead code: commented-out ticket routes + axum 0.6 path syntax | `server.rs:27`, `router/tickets.rs:14-21`, `controller/{ticket,tickets}.rs` | Delete or re-enable with 0.7 syntax (`{id}` not `:id`) |
| Q3 | Duplication: two routing architectures | `router/` vs `rpc/` | Pick one (see [architecture.md](architecture.md) § 9.1) |
| Q4 | Dev code in prod path | `main.rs:17-18`, `server.rs:16` | Feature-gate `_dev_utils` (S7) |
| Q5 | Error inconsistency: three error types | `rpc/mod.rs` vs `error/mod.rs` vs `model/error.rs` | Uncomment `From<model::error::Error>`; unify; map unused variants |
| Q6 | No-op tests — missing `#[tokio::test]`/`#[test]` | `user.rs:77-90`, `task.rs:75-99`, `crypt/mod.rs:35-52`, `crypt/token.rs:99-108` | Add test attributes; `httpc-test` declared but unused |
| Q7 | Naming typos | `PwdNotMathing`, `FailtToB643UrlDecode`, `UnkownRpcMethod`, `logff` | Rename |
| Q8 | Naming/schema drift | `name` ↔ `username`; `title` ↔ `name`; `tasks` ↔ `task` | Align (see [data-models.md](data-models.md)) |
| Q9 | Dropped `Result` | `dev_db.rs:58` (`update_pwd`), `observability::startup()`, `init_dev()` | Propagate or log |
| Q10 | Missing HTTP-layer tests | `httpc-test` unused | Add handler/middleware/RPC tests |

---

## 3. Test Coverage

### Running tests (4 tests)

All in `service/task.rs`, all `#[serial] #[tokio::test]`, all require a live Postgres + `init_dev()` seeding:

| Test | Asserts |
|---|---|
| `test_create` | Task creation returns id |
| `test_list` | List returns created tasks |
| `test_get_err_not_found` | Missing id → `EntityNotFound` |
| `test_delete_err_not_found` | Deleting missing id → `EntityNotFound` |

### Tests that never run (4 functions missing attributes)

| Test | File | Issue |
|---|---|---|
| `test_first_ok_demo1` | `service/user.rs:77-90` | No `#[tokio::test]` |
| `test_update` | `service/task.rs:75-99` | No test attributes |
| `test_encrypt_into_b64url` | `crypt/mod.rs:35-52` | No `#[test]` |
| `test_generate_web_token` | `crypt/token.rs:99-108` | No `#[test]`; print-only stub |

### Missing coverage (0%)

- `crypt`: pwd encrypt/validate, token sign/verify/expiry/tamper
- `middleware`: ctx resolver, require_auth, cookie refresh
- `controller`: login/logoff handlers
- `rpc`: dispatch (valid/invalid, malformed params)
- `middleware/response`: response mapper
- `config`: env loading (missing/malformed)
- Security/race: user enumeration, tenancy/IDOR, rate-limiting

**Effective coverage ≈ 15%** of security-critical code.

---

## 4. OWASP Top 10 (2021) Checklist

| Category | Status | Notes |
|---|---|---|
| A01 Broken Access Control | ❌ **FAIL** | `_ctx` ignored; IDOR (S8) |
| A02 Cryptographic Failures | ❌ **FAIL** | HMAC-SHA512 not KDF (S3); hardcoded keys (S2); no TLS/`Secure` cookie (S6); non-constant-time (S3/S11) |
| A03 Injection | ✅ **Pass*** | `sqlb` parameterized, no string-concatenated queries; *dev `pexec` raw (S14) |
| A04 Insecure Design | ❌ **FAIL** | Auth `todo!()` + missing ext insert (S1); in-memory ticket; no rate limit (S5) |
| A05 Security Misconfiguration | ❌ **FAIL** | Dev utils in prod (S7); root Docker (S10); debug logging; missing runtime secrets |
| A07 Identification & Auth Failures | ❌ **FAIL** | Auth broken (S1); brute-force unlimited (S5); cookie attrs (S6); `welcome` demo creds; logout without revocation; auth errors → 500 (S9) |
| A08 Software & Data Integrity | ❌ **FAIL** | Unpinned images; no `cargo audit`/`cargo-deny` in CI; no SBOM; secrets in build context; `Cargo.lock` gitignored |
| A09 Security Logging | ⚠️ **Partial** | `req_uuid` good; `debug!`-only; internal error payloads in logs; no auth-failure/audit events |

---

## 5. Remediation Roadmap

### P0 — Must fix before any deployment

| Finding | Effort | Action |
|---|---|---|
| **S1** — auth non-functional | S | Implement `_generate_token`; insert CTX into extensions; fix `validate_pwd` content (see [auth-flow.md](auth-flow.md)) |
| **S2** — secrets committed | S | Rotate; remove from `.cargo/config.toml`; externalize; purge history; add secret scanning to CI |
| **S3** — HMAC-SHA512 not KDF | M | Replace with argon2id; constant-time verify; `#01#` migration on next login |
| **S4** — panic on request path | M | Propagate `Result`; JSON-RPC error envelope; `clippy::unwrap_used` CI gate |

### P1 — Fix before exposing any endpoint

| Finding | Effort | Action |
|---|---|---|
| **S7** — dev DB resetter in prod | S | Feature-gate `_dev_utils` |
| **S8** — IDOR / no ownership | M | Add `cid` column; scope queries by `ctx.user_id()` |
| **S15** — salt columns on wrong table | M | Reconcile schema; adopt `sqlx::migrate` |
| **S5** — no rate limiting | M | `tower_governor`; lockout |
| **S6** — cookie attrs + revocation | S | `Secure`, `SameSite`, `Max-Age`; rotate `token_salt` on logoff; absolute lifetime |

### P2 — Hardening & cleanup

| Finding | Effort | Action |
|---|---|---|
| **S9** — 4xx/5xx mapping | S | Explicit arms; anti-enumeration |
| **S10** — Docker hardening | M | See [deployment-operations.md](deployment-operations.md) |
| **Q1–Q10** | M | Dead code, typos, no-op test repairs, clippy gate, `cargo audit`, `deny_unknown_fields` |

### P3 — Defense in depth

| Finding | Effort | Action |
|---|---|---|
| **S11** | S–M | Constant-time verify; `user.id` in token ident; remove `unwrap()`s |
| **S12** | M | Security headers; TLS; CORS |
| **S13** | S–M | Input validation; body size limit |
| **S14** | S | `sqlx::migrate`; fix password typo |

---

## 6. Bottom line

> The architecture is a solid base for a learning project — layered, dependency-injected via axum `State`, with a single error-mapping point and per-request `req_uuid`. But it is **not shippable**. Fix P0 → P1 → re-run the full test matrix (including new HTTP tests for auth) → then consider deployment.
