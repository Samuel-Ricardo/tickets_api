use axum::{routing::post, Router};

use crate::{controller::login::api_login_handler, model::ModelManager};

pub fn routes(manager: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .with_state(manager)
}
