use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use crate::{
    crypt::{pwd, token::generate_web_token, EncryptContent},
    ctx::CTX,
    error::{Error, Result},
    middleware::auth::AUTH_TOKEN,
    model::{login::LoginPayload, user::UserForLogin, ModelManager},
    service::user::UserService,
};

pub fn set_token_cookies(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());

    cookie.set_http_only(true);
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(())
}

pub fn remove_token_cookies(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::named(AUTH_TOKEN);
    cookie.set_path("/");
    cookies.remove(cookie);
    Ok(())
}

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

    set_token_cookies(&cookies, &user.name, &user.token_salt.to_string())?;

    let body = Json(json!({
        "result": {
        "success": true
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
pub struct LogoffPayload {
    pub logff: bool,
}

pub async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!(" {:<12} - api_logoff", "HANDLER");

    let should_logoff = payload.logff;

    if should_logoff {
        remove_token_cookies(&cookies)?;
    }

    let body = Json(json!({
        "result": {
        "logged off": should_logoff
        }
    }));

    Ok(body)
}
