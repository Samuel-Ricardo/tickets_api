pub mod login;

use axum::{routing::get, Router};

use crate::controller::hello_handler;

pub async fn hello_router() -> Router {
    Router::new().route("/", get(hello_handler))
}
