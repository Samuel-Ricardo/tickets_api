use sqlb::HasFields;
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
    E: HasFields,
{
    let db = manager.db();

    //    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    //
    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &CTX, manager: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = manager.db();

    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn create<MC, E>(_ctx: &CTX, manager: &ModelManager, entity: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = manager.db();
    let fields = entity.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
}

pub async fn update<MC, E>(_ctx: &CTX, manager: &ModelManager, id: i64, entity: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = manager.db();

    let count = sqlb::update()
        .table(MC::TABLE)
        .data(entity.not_none_fields())
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(id)
    }
}
