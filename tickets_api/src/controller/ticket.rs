use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Error, Result};

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

    pub async fn list_tickets(&self, _ctx: CTX) -> Result<Vec<Ticket>> {
        let store = self.tickets_store.lock().await;

        let tickets = store.iter().filter_map(|t| t.clone()).collect();

        Ok(tickets);
    }

    pub async fn delete_ticket(&self, _ctx: CTX, id: u64) -> Result<Ticket> {
        let mut store = self.tickets_store.lock().await;

        let ticket = store.get_mut(id as usize).and_then(|t| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}
