use axum::{
    extract::{Path, State},
    Json,
};
use tracing::debug;

use crate::{
    ctx::CTX,
    error::Result,
    model::ticket::{Ticket, TicketForCreate},
};

use super::ticket::TicketController;

pub async fn create_ticket(
    State(controller): State<TicketController>,
    ctx: CTX,
    Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    debug!(" {:<12} - create_ticket", "HANDLER");

    let ticket = controller.create_ticket(ctx, ticket_fc).await?;

    Ok(Json(ticket))
}

pub async fn delete_ticket(
    State(controller): State<TicketController>,
    ctx: CTX,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
    debug!("{:<12} - delete_ticket", "HANDLER");

    let ticket = controller.delete_ticket(ctx, id).await?;

    Ok(Json(ticket))
}

pub async fn list_tickets(
    State(controller): State<TicketController>,
    ctx: CTX,
) -> Result<Json<Vec<Ticket>>> {
    debug!(" {:<12} - get_ticket", "HANDLER");

    let ticket = controller.list_tickets(ctx).await?;

    Ok(Json(ticket))
}
