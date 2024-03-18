use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use tower_cookies::Cookies;

use crate::controller::ticket::TicketController;
use crate::error::Error;
use crate::{ctx::CTX, error::Result};

pub const AUTH_TOKEN: &str = "auth-token";

pub async fn mw_require_auth(ctx: Result<CTX>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    _mc: State<TicketController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => Ok(CTX::new(user_id)),
        Err(e) => Err(e),
    };
}

impl<S: Send + Sync> FromRequestParts<S> for CTX {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - CTX", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<CTX>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone();
    }
}

fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_wholem, user_id, exp, sign) = regex_captures!(
        r#"^user-(\d+)\.(.+)\.(.+)"#, // a literal regex
        &token
    )
    .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
