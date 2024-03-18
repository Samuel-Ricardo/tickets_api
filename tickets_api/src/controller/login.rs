use axum::Json;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{
    error::{Error, Result},
    middleware::auth::AUTH_TOKEN,
    model::login::LoginPayload,
};

async fn api_login_handler(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    if payload.username != "samuel" || payload.pwd != "123" {
        return Err(Error::LoginFail);
    }

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
