use axum::{
    middleware as mw,
    routing::{delete, get, post, Router},
};

use crate::{
    controller::{
        ticket::TicketController,
        tickets::{create_ticket, delete_ticket, list_tickets},
    },
    middleware,
};

pub fn routes(controller: TicketController) -> Router {
    Router::new()
        .route("/ticket", post(create_ticket))
        .route("/ticket/:id", delete(delete_ticket))
        .route("/tickets", get(list_tickets))
        .with_state(controller)
        .route_layer(mw::from_fn(middleware::auth::mw_require_auth))
}
