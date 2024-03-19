pub mod login;
pub mod tickets;

use axum::{routing::get, Router};

use crate::controller::hello_handler;

pub fn hello_router() -> Router {
    Router::new().route("/", get(hello_handler))
}
