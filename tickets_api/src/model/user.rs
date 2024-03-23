use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::{postgres::PgRow, prelude::FromRow};
use uuid::Uuid;

#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
    pub name: String,
    pub pwd_clear: String,
}
