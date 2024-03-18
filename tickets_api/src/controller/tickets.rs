use axum::{extract::State, Json};

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
    println!("->> {:<12} - create_ticket", "HANDLER");

    let ticket = controller.create_ticket(ctx, ticket_fc).await?;

    Ok(Json(ticket))
}
