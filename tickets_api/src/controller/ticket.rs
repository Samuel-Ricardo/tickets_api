use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::Result;

use crate::{
    ctx::CTX,
    model::ticket::{Ticket, TicketForCreate},
};

#[derive(Clone)]
pub struct TicketController {
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

/* Constructor */
impl TicketController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }
}

impl TicketController {
    pub async fn create_ticket(&self, ctx: CTX, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().await;

        let id = store.len() as u64;
        let ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };

        store.push(Some(ticket.clone()));

        Ok(ticket)
    }
}
