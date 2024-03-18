use axum::{routing::post, Router};

use crate::controller::login::api_login_handler;

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login_handler))
}
