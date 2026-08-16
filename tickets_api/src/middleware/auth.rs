use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use crate::controller::login::set_token_cookies;
use crate::controller::ticket::TicketController;
use crate::crypt::token::{validate_web_token, Token};
use crate::error::Error;
use crate::model::user::UserForAuth;
use crate::model::ModelManager;
use crate::service::user::UserService;
use crate::{ctx::CTX, error::Result};

use async_trait::async_trait;

pub const AUTH_TOKEN: &str = "auth-token";

pub async fn mw_require_auth(ctx: Result<CTX>, req: Request<Body>, next: Next) -> Result<Response> {
    debug!(" {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    //    controller: State<TicketController>,
    manager: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!(" {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let ctx_result = _ctx_resolve(manager, &cookies).await;

    if ctx_result.is_err() && !matches!(ctx_result, Err(Error::TokenNotInCookie)) {
        cookies.remove(Cookie::named(AUTH_TOKEN));
    }

    Ok(next.run(req).await)
}

async fn _ctx_resolve(manager: State<ModelManager>, cookies: &Cookies) -> Result<CTX> {
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(Error::CtxExtractFail)?;

    let token: Token = token.parse().map_err(|_| Error::AuthFailTokenWrongFormat)?;

    let user: UserForAuth =
        UserService::first_by_username(&CTX::root_ctx(), &manager, &token.ident)
            .await
            .map_err(|_| Error::ServiceAccessError)?
            .ok_or(Error::UserNotFound)?;

    validate_web_token(&token, &user.token_salt.to_string()).map_err(|_| Error::ValidationFail)?;

    set_token_cookies(cookies, &user.name, &user.token_salt.to_string())?;

    Ok(CTX::new(user.id as u64).map_err(|ex| Error::CtxCreationFail(ex.to_string()))?)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CTX {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!(" {:<12} - CTX", "EXTRACTOR");

        parts
            .extensions
            .get::<Result<CTX>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
        //            .map_err(Error::CtxExtractFail)
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
