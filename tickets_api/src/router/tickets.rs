use axum::{
    middleware as mw,
    routing::{delete, post, Router},
};

use crate::{
    controller::{
        ticket::TicketController,
        tickets::{create_ticket, delete_ticket},
    },
    middleware,
};

pub fn routes(controller: TicketController) -> Router {
    Router::new()
        .route("/ticket", post(create_ticket))
        .route("/ticket/:id", delete(delete_ticket))
        .with_state(controller)
        .route_layer(mw::from_fn(middleware::auth::mw_require_auth))
}
