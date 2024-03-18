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
