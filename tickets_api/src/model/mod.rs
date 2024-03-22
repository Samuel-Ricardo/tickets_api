use crate::service;

use self::store::{new_db_pool, DB};

pub mod error;
pub mod login;
pub mod store;
pub mod task;
pub mod ticket;

use error::Result;

#[derive(Clone)]
pub struct ModelManager {
    db: DB,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    // INFO: Returns the SQLX DB Pool Reference [Only for the Model Layer]
    //    pub(in super::model) fn db(&self) -> &DB {
    pub fn db(&self) -> &DB {
        &self.db
    }
}
