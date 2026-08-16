# API Reference

The HTTP API surface of `tickets_api` (latest `v2`). All endpoints are mounted in `src/server.rs`. Paths are relative to the server root (`0.0.0.0:8080`).

> ⚠️ Several endpoints are currently non-functional due to the P0 blockers (login panics; every authenticated request fails). The contract below documents the *intended* behavior alongside the *actual* broken behavior.

## Endpoints

| Method | Path | Handler | Auth | Mounted |
|---|---|---|---|---|
| GET | `/` | `controller::hello_handler` | No | ✅ |
| POST | `/api/login` | `controller::api_login_handler` | No | ✅ |
| POST | `/api/logoff` | `controller::api_logoff_handler` | No | ✅ |
| POST | `/api/rpc` | `rpc::rpc_handler` | **Yes** (`mw_require_auth` route_layer) | ✅ |
| POST | `/api/ticket` | `controller::create_ticket` | Yes | ❌ commented out |
| DELETE | `/api/ticket/:id` | `controller::delete_ticket` | Yes | ❌ commented out |
| GET | `/api/tickets` | `controller::list_tickets` | Yes | ❌ commented out |
| * | `/*` | `router::statics()` → `ServeDir` | No | ✅ (fallback) |

## `GET /`

Returns a hardcoded hello.

```http
GET / HTTP/1.1
```
```http
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8

{"hello":"world"}
```
(Emitted as a raw `Response<String>` with a 200 — the body is the literal JSON-ish string.)

## `POST /api/login`

Authenticate and set the `auth-token` cookie.

**Request body** — `application/json`:
```json
{ "username": "demo1", "pwd": "welcome" }
```
where `LoginPayload { username: String, pwd: String }`.

**Flow** (`controller/login.rs::api_login_handler`):
1. Root ctx → `UserService::first_by_username` → `UserForLogin` (else `LoginFailUsernameNotFound`).
2. Unwrap `user.pwd` (else `LoginFailUserHashNoPwd`).
3. `pwd::validate_pwd(...)` — ⚠️ **currently validates the stored hash against itself**; the payload `pwd` is unused, so this always yields `PwdNotMatching` → `LoginFailPwdNotMathing`. (A correct flow would use `content: payload.pwd`.)
4. `set_token_cookies` → `generate_web_token` → ⚠️ `_generate_token` is `todo!()` → **runtime panic**. The connection is aborted; no response is returned.

**Intended response** (once fixed):
```http
HTTP/1.1 200 OK
set-cookie: auth-token=<token>; Path=/; HttpOnly
content-type: application/json

{"result":{"success":true}}
```

> Because of the panic, this endpoint currently returns **no clean response** — the handler panics and the connection is dropped.

## `POST /api/logoff`

Remove the `auth-token` cookie.

**Request body** — `application/json`:
```json
{ "logff": true }
```
where `LogoffPayload { logff: bool }` (note the field name typo `logff`).

**Response**:
```http
HTTP/1.1 200 OK
content-type: application/json

{"result":{"logged off": true}}
```
The cookie is removed only when `logff` is truthy. **Server-side token revocation is not implemented** — the cookie is deleted client-side but the token remains valid (no `token_salt` rotation on logoff) — see [security-review.md](security-review.md) S6.

## `POST /api/rpc`

Generic JSON-RPC-style dispatcher (auth-gated). Request body — `application/json` of `RpcRequest`:

```json
{
  "id": "req-1",
  "method": "create_task",
  "params": { "data": { "title": "Buy milk" } }
}
```

`RpcRequest { id: Option<Value>, method: String, params: Option<Value> }`. The `id` is echoed back in the response.

### RPC methods

| `method` | `params` | Success `result` |
|---|---|---|
| `create_task` | `{"data":{"title": String}}` | `{"id": i64, "title": String}` (re-fetched) |
| `list_tasks` | *(omit params)* | `[{"id": i64, "title": String}]` |
| `update_task` | `{"id": i64, "data":{"title": String?}}` | `{"id": i64, "title": String}` (re-fetched) |
| `delete_task` | `{"id": i64}` | `{"id", "title"}` (fetched before delete) |
| *(unknown method)* | — | **panic** — `Err(Error::UnkownRpcMethod(method)).unwrap()` |

### Examples

Create a task:
```http
POST /api/rpc HTTP/1.1
cookie: auth-token=<token>
content-type: application/json

{"id":"r1","method":"create_task","params":{"data":{"title":"Write docs"}}}
```
```json
{ "id": "r1", "result": { "id": 1000, "title": "Write docs" } }
```

List tasks:
```json
{"id":"r2","method":"list_tasks"}
```
```json
{ "id": "r2", "result": [ { "id": 1000, "title": "Write docs" } ] }
```

### Error behavior (current — broken)

The RPC layer **panics instead of returning a JSON-RPC error**:
- Unknown method → `Err(Error::UnkownRpcMethod).unwrap()` → panic.
- Missing/malformed `params` → `from_value(...).unwrap()` → panic.
- Any service error (`EntityNotFound`, pool failure, …) → panic via the `exec_rpc_fn!` macro's `.unwrap().unwrap()`.

**Recommendation:** emit a JSON-RPC error object: `{"id": id, "error": {"code": -32600, "message": "..."}}`; see [security-review.md](security-review.md) S4 and [issue-roadmap.md](issue-roadmap.md).

### Auth note

`/api/rpc` is gated by `mw_require_auth` (a route_layer on the `/api` subtree). `mw_require_auth` extracts `ctx: Result<CTX>` from extensions (populated by `mw_ctx_resolver`). Because `mw_ctx_resolver` **never inserts the resolved CTX into extensions**, the extractor always finds `None` → `AuthFailCtxNotInRequestExt` → **403 `NO_AUTH` for every request** (P0-1). There is no path that currently yields a 200 from `/api/rpc`.

## Error envelope

Application errors (where they are returned rather than panicked) are normalized by `main_response_mapper` (`middleware/response.rs`) into:

```json
{
  "error": {
    "type": "NO_AUTH",
    "req_uuid": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```
- `type` is the `strum AsRefStr` of the `ClientError` (`LOGIN_FAIL`, `NO_AUTH`, `INVALID_PARAMS`, `SERVICE_ERROR`).
- `req_uuid` is a fresh `Uuid::new_v4()` per request, also present in the request log line.

### Status mapping (`Error::client_status_and_error`)

| `Error` variant | HTTP | `ClientError` |
|---|---|---|
| `LoginFail` | 403 | `LOGIN_FAIL` |
| `AuthFailNoAuthTokenCookie`, `AuthFailTokenWrongFormat`, `AuthFailCtxNotInRequestExt` | 403 | `NO_AUTH` |
| `TicketDeleteFailIdNotFound { id }` | 400 | `INVALID_PARAMS` |
| *everything else* (token errors, crypt errors, config errors, RPC errors, `UserNotFound`, `ValidationFail`, `TokenExpired`, …) | 500 | `SERVICE_ERROR` |

> The large default arm is a known weakness — see [security-review.md](security-review.md) S9 and [issue-roadmap.md](issue-roadmap.md).
