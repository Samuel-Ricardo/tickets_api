# ADR-003: Custom Signed-Cookie Token over JWT Bearer

- **Status:** Accepted (blocked by S1/S6 — currently inoperative in v2)
- **Date:** 2026-08-14 (recorded during review; decision predates)
- **Related:** [auth-flow.md](../auth-flow.md), [security-review.md](../security-review.md) S1/S3/S6/S11

## Context

The API needed authenticated client state that survives across requests without a server-side session store. Standard choices: opaque session IDs in a store, JWT bearer tokens, or a signed cookie. The project chose a **custom opaque signed cookie** to avoid extra dependencies (jwt crate) and to keep auth stateless.

## Decision

Issue a cookie `auth-token` whose value is a custom token:

```
b64u(ident) "." b64u(exp) "." sign_b64u
```

- `ident` = username; `exp` = RFC 3339 UTC expiry (`SERVICE_TOKEN_DURATION_SEC`, 1800 s)
- `sign_b64u` = `b64url(HMAC-SHA512(TOKEN_KEY, b64u(ident) "." b64u(exp) ‖ token_salt))`
- `token_salt` = per-user UUID enabling immediate per-user revocation
- Cookie: `HttpOnly`, `Path=/` (Secure/SameSite pending S6)

## Consequences

**Positive**
- Stateless validation — no session table, revocation via `token_salt` rotation (intended)
- Smaller dependency surface (no JWT crate), full control of claims
- Cookie semantics make browser integration simple (`HttpOnly`)

**Negative**
- **Blocked in v2:** `_generate_token` is `todo!()` → login panics (S1)
- Custom crypto invites mistakes: non-constant-time compare (S11), no `iss/aud/jti/nonce` claims, username PII in the cookie (S11)
- Key redistribution: if `TOKEN_KEY` leaks (S2), all tokens are forgeable
- Logoff doesn't rotate `token_salt` — no real server-side invalidation (S6)

## Alternatives considered

- **JWT bearer (Authorization header)** — JSON-Web-Token standard, but adds a dependency, exposes claims to callers, and requires client-side storage/rotation logic
- **Opaque session tokens in DB** — best revocation/rotation story, but adds a session table and DB hit per request; the project already does a per-request DB user lookup in `mw_ctx_resolver`

## Related work

- P0-2: implement `_generate_token`
- P1-5: `Secure`/`SameSite`/`Max-Age`, absolute lifetime cap, `token_salt` rotation on logoff