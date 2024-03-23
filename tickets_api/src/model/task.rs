use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Fields)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

/* -- DTO -- */
#[derive(Deserialize, Fields)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Deserialize, Fields)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}
