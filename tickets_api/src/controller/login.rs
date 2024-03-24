use axum::{extract::State, Json};
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use crate::{
    crypt::{pwd, EncryptContent},
    ctx::CTX,
    error::{Error, Result},
    middleware::auth::AUTH_TOKEN,
    model::{login::LoginPayload, user::UserForLogin, ModelManager},
    service::user::UserService,
};

pub async fn api_login_handler(
    State(manager): State<ModelManager>,
    cookies: Cookies,
    payload: Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!(" {:<12} - api_login", "HANDLER");

    let payload = LoginPayload {
        username: payload.username.to_string(),
        pwd: payload.pwd.to_string(),
    };
    let root_ctx = CTX::root_ctx();

    let user: UserForLogin = UserService::first_by_username(&root_ctx, &manager, &payload.username)
        .await
        .unwrap()
        .ok_or(Error::LoginFailUsernameNotFound)?;

    let user_id = user.id;

    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHashNoPwd { user_id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            salt: user.pwd_salt.to_string(),
            content: pwd.clone(),
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMathing { user_id })?;

    let mut cookie = Cookie::new(AUTH_TOKEN, "user-1.exp.sign");
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    let body = Json(json!({
        "result": {
        "success": true
        }
    }));

    Ok(body)
}
