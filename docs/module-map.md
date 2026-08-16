# Module Map

Type-level map of `src/` (all paths relative to `tickets_api/src/`). 46 files across 20 directories. Derives from the latest `v2` state.

## `main.rs`
- Binary entry point. Declares 15 modules. `pub use config::config;` `pub use error::{Error, Result};`.
- `pub mod _dev_utils;` — **not** `#[cfg(test)]`-gated → dev-seeding code ships in release builds (P0-3).
- `#[tokio::main] async fn main() -> Result<()>` → `server::startup().await`.

## `server.rs`
- `pub async fn startup() -> Result<()>` — entire bootstrap: `observability::startup()`; `let _ = _dev_utils::init_dev()` (swallowed); `ModelManager::new().await.unwrap()`; `TicketController::new().await.unwrap()`; route assembly; layers (`CookieManagerLayer`, `mw_ctx_resolver` `from_fn_with_state`, `main_response_mapper` `map_response`); `mw_require_auth` route_layer on `/api`; fallback `router::statics()`; bind `0.0.0.0:8080`; `axum::serve(...).await.unwrap()` (**no graceful shutdown**).

## `ctx.rs`
- `pub struct CTX { user_id: u64 }` (Clone, Debug). `CTX::new(u64) -> Result<Self>` (rejects 0 → `CannotNewRootCtx`). `CTX::root_ctx()` (user_id 0). `CTX::user_id(&self) -> u64`.

## `config/mod.rs`
- `pub fn config() -> &'static Config` — `OnceLock<Config>` singleton; `panic!("FATAL - WHILE LOADING CONFIG")` on error.
- `pub struct Config { DB_URL, WEB_FOLDER: String; PWD_KEY, TOKEN_KEY: Vec<u8>; TOKEN_DURATION_SEC: f64 }` (`#[allow(non_snake_case)]`).
- Private: `load_from_env`, `get_env`, `get_env_b64u_as_u8s`, `get_env_parse<T: FromStr>`.

## `base/`
- `db.rs` — `pub trait DbBmc { const TABLE: &'static str; }`; generic async CRUD over `sqlb`:
  - `get<MC,E>(&CTX, &ModelManager, id: i64) -> Result<E>` (`Err EntityNotFound`)
  - `list<MC,E>(&CTX, &ModelManager) -> Result<Vec<E>>`
  - `create<MC,E>(&CTX, &ModelManager, E) -> Result<i64>` (`returning("id")`)
  - `update<MC,E>(&CTX, &ModelManager, i64, E) -> Result<()>` (0 rows → `EntityNotFound`)
  - `delete<MC>(&CTX, &ModelManager, i64) -> Result<()>` (0 rows → `EntityNotFound`)
  - `&CTX` is taken but ignored (`_ctx`) — see S8 IDOR.

## `controller/`
- `mod.rs` — `pub async fn hello_handler() -> Response<String>` (`{"hello":"world"}`, 200).
- `login.rs` — `set_token_cookies(cookies, user, salt) -> Result<()>`; `remove_token_cookies(cookies) -> Result<()>`; `api_login_handler(State<ModelManager>, Cookies, Json<LoginPayload>) -> Result<Json<Value>>`; `LogoffPayload { logff: bool }`; `api_logoff_handler`.
- `ticket.rs` — **legacy in-memory** `TicketController { tickets_store: Arc<Mutex<Vec<Option<Ticket>>>> }` (Clone); `new()`, `create_ticket`, `list_tickets`, `delete_ticket` (id = `store.len()`, `Option::take()`).
- `tickets.rs` — axum handlers `create_ticket`/`delete_ticket`/`list_tickets` wrapping `TicketController`.

## `crypt/`
- `mod.rs` — `EncryptContent { content, salt }`; `encrypt_into_b64url(key, &enc) -> Result<String>` = HMAC-SHA512(key, content+salt) → b64url. One `#[cfg(test)]` determinism test (**missing attribute** — never runs).
- `error.rs` — `enum Error { KeyFailHmac, PwdNotMatching }` (Debug, Serialize/Deserialize, Clone; `Display` via `{self:?}`; `std::error::Error`).
- `pwd.rs` — `encrypt_pwd(&enc) -> Result<String>` (prefixes `"#01#"`); `validate_pwd(&enc, pwd_ref) -> Result<()>` (equality compare → `PwdNotMatching`).
- `token.rs` — `Token { ident, exp, sign_b64u }`; `_generate_token(...)` = **`todo!()`**; `_validate_token_sign_and_exp`; `_token_sign_into_b64url`; `generate_web_token(user, salt) -> Result<Token>` (calls the `todo!()`); `validate_web_token`; `FromStr` (split `.` → 3 parts); `Display`.

## `error/mod.rs`
- `pub type Result<T> = Result<T, Error>;`
- `enum Error` — **21 variants**: `LoginFail`; `AuthFail{NoAuthTokenCookie,TokenWrongFormat,CtxNotInRequestExt}`; `TicketDeleteFailIdNotFound{id:u64}`; `ConfigMissingEnv`, `ConfigWrongFormat`; `CannotNewRootCtx`, `CtxExtractFail`, `CtxCreationFail(String)`; `LoginFailUsernameNotFound`, `LoginFailUserHashNoPwd{user_id}`, `LoginFailPwdNotMathing{user_id}`; `DateFailParse(String)`, `FailtToB643UrlDecode`; `TokenInvalidFormat`, `TokenCannotDecodeIdent`, `TokenCannotDecodeExp`, `TokenSignatureNotMatching`, `TokenExpNotIso`, `TokenExpired`; `Crypt(crypt::Error)`; `UserNotFound`, `ServiceAccessError`, `ValidationFail`, `TokenNotInCookie`; `UnkownRpcMethod(String)`, `RpcMissingParams`, `RpcFailJsonParams`.
- `From<crypt::Error>` exists; `From<model::error::Error>` **commented out**.
- `IntoResponse for Error` → 500 + insert self into extensions.
- `client_status_and_error()` → see [api-reference.md](api-reference.md) status map.
- `enum ClientError { LOGIN_FAIL, NO_AUTH, INVALID_PARAMS, SERVICE_ERROR }` (`AsRefStr`).

## `log/mod.rs`
- `log_request(uuid, method, uri, ctx: Option<CTX>, service_error, client_error) -> Result<()>` — one `debug!` JSON line `RequestLogLine { uuid, timestamp epoch-ms, user_id, req_path, req_method, client_error_type, error_type, error_data }` (`#[skip_serializing_none]`).

## `middleware/`
- `auth.rs` — `AUTH_TOKEN = "auth-token"`; `mw_require_auth(ctx: Result<CTX>, req, next)` (fails fast via `ctx?`); `mw_ctx_resolver(State<ModelManager>, Cookies, mut req, next)` (computes `ctx_result` but **never inserts into `req.extensions`** → P0-1); `_ctx_resolve` (cookie → `Token::from_str` → `UserService::first_by_username(token.ident)` → `validate_web_token` → re-issue cookie (sliding) → `CTX::new(user.id)`; removes cookie on error except `TokenNotInCookie`); `FromRequestParts<S> for CTX` (reads `Result<CTX>` from extensions → else `AuthFailCtxNotInRequestExt`); **dead**: `parse_token` regex fn.
- `response.rs` — `main_response_mapper(ctx: Option<CTX>, uri, method, res)` via `map_response`: `Uuid::new_v4()` per request, extract `Error` from extensions, `client_status_and_error()`, body `{"error":{"type","req_uuid"}}`, `log_request`, `debug!` client_error_body.

## `model/`
- `mod.rs` — `ModelManager { db: DB }` (Clone); `new() -> store::new_db_pool`; `db(&self) -> &DB`. (Unused `use crate::service;`.)
- `error.rs` — `enum Error { Crypt, Store, Sqlx(DisplayFromStr), EntityNotFound{entity,id} }` (`From` impls; **no Display/std::error::Error**).
- `login.rs` — `LoginPayload { username, pwd }`.
- `user.rs` — `User { id, name }`; `UserForCreate { name, pwd_clear }`; `UserForInsert` (private, unused); `UserForLogin { id, name, pwd, pwd_salt, token_salt }`; `UserForAuth { id, name, token_salt }`; trait `UserBy` (blanket impls for `User`, `UserForLogin`, `UserForAuth`).
- `ticket.rs` — `Ticket { id:u64, cid, title }`; `TicketForCreate { title }` (no DB mapping).
- `task.rs` — `Task { id:i64, title }`; `TaskForCreate { title }`; `TaskForUpdate { title: Option<String> }`.
- `store/mod.rs` — `type DB = Pool<Postgres>;`; `new_db_pool()` — `PgPoolOptions::max_connections(5).connect(config().DB_URL)` (→ `FailToCreatePool`).
- `store/error.rs` — `enum Error { FailToCreatePool(String) }` (has Display+std::error::Error).

## `observability/mod.rs`
- `startup()` — `tracing_subscriber::fmt().without_time().with_target(false).with_env_filter(EnvFilter::from_default_env()).init()`.

## `router/`
- `mod.rs` — `hello_router()` (`GET /`); `statics()` → `nest_service("/", ServeDir(config().WEB_FOLDER))`.
- `login.rs` — `routes(manager)` → `POST /api/login`, `POST /api/logoff` `.with_state(manager)`.
- `tickets.rs` — `routes(controller)` → `POST /api/ticket`, `DELETE /api/ticket/:id` (axum 0.6 syntax), `GET /api/tickets` with `route_layer(mw_require_auth)`. **Not mounted** (commented out in `server.rs`).

## `rpc/`
- `mod.rs` — `macro_rules! exec_rpc_fn!` (`.map(to_value).unwrap().unwrap()` → panics); `rpc_handler(State<ModelManager>, ctx: CTX, Json<RpcRequest>) -> Response` — dispatch `create_task`/`list_tasks`/`update_task`/`delete_task`, else `Err(UnkownRpcMethod).unwrap()` → panic; success → `{"id","result"}`.
- `model/mod.rs` — `RpcRequest`, `ParamsForCreate<D>`, `ParamsForUpdate<D>`, `ParamsIded`.
- `router/mod.rs` — `main(manager)` → `POST /api/rpc` `.with_state(manager)`.
- `task/mod.rs` — thin wrappers `create_task`, `list_tasks`, `update_task`, `delete_task` (all `.unwrap()` on errors; write→re-`get`→return full entity).

## `service/`
- `user.rs` — `UserService; impl DbBmc { TABLE="user" }`; `get<E: UserBy>`; `first_by_username<E: UserBy>` (`WHERE name = $username` — queries `name`, schema has `username`); `update_pwd`. `#[cfg(test)] test_first_ok_demo1` (no `#[tokio::test]` → never runs).
- `task.rs` — `TaskService; impl DbBmc { TABLE="tasks" }` (plural vs schema `task`); CRUD pass-throughs to `base::db`. `#[cfg(test)]` `test_create`/`test_list`/`test_get_err_not_found`/`test_delete_err_not_found` (`#[serial] #[tokio::test]` — run); `test_update` (no attrs — never runs).

## `util/`
- `base64.rs` — `b64u_encode`, `b64u_decode` (error `FailtToB643UrlDecode` typo).
- `time.rs` — `now_utc`, `format_time` (RFC3339, `unwrap`), `now_utc_plus_sec_str` (unused), `parse_utc` (`DateFailParse`).

## `_dev_utils/` (unconditionally public)
- `mod.rs` — `init_dev()` (`OnceCell<()>` → `dev_db::init_dev_deb`); `init_test_db() -> ModelManager` (cached `OnceCell`).
- `dev_db.rs` — `PG_DEV_POSTGRES_URL = postgres://postgres:welcome@db:5432/postgres`, `PG_DEV_APP_URL = ...db_test:5432/app_db`, `SQL_RECREATE_DB`, `SQL_DIR`, `DEMO_PWD="welcome"`; `init_dev_deb()` drop/recreate + run SQL (sorted, split on `;`); `pexec()` naive `;` split (FIXME); `new_db_pool` (max 1, 500ms).
- `key.rs` — `gen_random()` prints 64-byte key b64url.
- `seed.rs` — `task(ctx, manager, titles)` creates tasks.

## Empty / dead scaffolding
- `crates/libs/lib-auth/` — empty directory (no files).
- Dead code: `parse_token` (regex), `_generate_token` (`todo!()`), `now_utc_plus_sec_str`, `UserForInsert`, unused `TicketController` import in `middleware/auth.rs`.
