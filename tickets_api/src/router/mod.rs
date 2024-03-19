pub mod login;
pub mod tickets;

use axum::{
    routing::{get, get_service},
    Router,
};
use tower_http::services::ServeDir;

use crate::controller::hello_handler;

pub fn hello_router() -> Router {
    Router::new().route("/", get(hello_handler))
}

pub fn statics() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
