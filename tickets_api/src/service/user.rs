use crate::{
    base::{self, db::DbBmc},
    ctx::CTX,
    model::{
        error::Result,
        user::{User, UserBy},
        ModelManager,
    },
};

pub struct UserService;

impl DbBmc for UserService {
    const TABLE: &'static str = "user";
}

impl UserService {
    pub async fn get<E>(ctx: &CTX, manager: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::db::get::<Self, _>(ctx, manager, id).await
    }
}
