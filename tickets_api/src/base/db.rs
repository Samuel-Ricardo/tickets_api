use sqlx::{postgres::PgRow, FromRow};

use crate::{
    ctx::CTX,
    model::{
        error::{Error, Result},
        ModelManager,
    },
};

pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn get<MC, E>(_ctx: &CTX, manager: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = manager.db();

    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entity: E = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}
