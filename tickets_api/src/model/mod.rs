use self::store::{new_db_pool, DB};

pub mod error;
pub mod login;
pub mod store;
pub mod task;
pub mod ticket;

use error::Result;

pub struct ModelManager {
    db: DB,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager { db })
    }

    // INFO: Returns the SQLX DB Pool Reference [Only for the Model Layer]
    pub(in crate::model) fn db(&self) -> &DB {
        &self.db
    }
}
