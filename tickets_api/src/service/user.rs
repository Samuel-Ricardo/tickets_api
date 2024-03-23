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

    pub async fn first_by_username<E>(
        _ctx: &CTX,
        manager: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = manager.db();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("name", "=", username)
            .fetch_optional::<_, E>(db)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::_dev_utils;
    use serial_test::serial;

    async fn test_first_ok_demo1() -> Result<()> {
        let manager = _dev_utils::init_test_db().await;
        let ctx = CTX::root_ctx();
        let name = "demo1";

        let user: User = UserService::first_by_username(&ctx, &manager, name)
            .await
            .unwrap()
            .unwrap();
        //            .context("Should have user")?;

        assert_eq!(user.name, name);
        Ok(())
    }
}
