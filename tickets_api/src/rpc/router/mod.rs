use axum::{routing::post, Router};

use crate::model::ModelManager;

use super::rpc_handler;

pub fn main(manager: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(manager)
}
