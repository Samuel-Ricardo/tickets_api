use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64, /* createor user_id  */
    pub title: String,
}

/* Create Ticket DTO */
#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}
