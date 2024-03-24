use crate::{
    base::{self, db::DbBmc},
    crypt::{pwd, EncryptContent},
    ctx::CTX,
    model::{
        error::Result,
        user::{User, UserBy, UserForLogin},
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

    pub async fn update_pwd(
        ctx: &CTX,
        manager: &ModelManager,
        id: i64,
        pwd_clear: &str,
    ) -> Result<()> {
        let db = manager.db();

        let user: UserForLogin = Self::get(ctx, manager, id).await?;
        let pwd = pwd::encrypt_pwd(&EncryptContent {
            content: pwd_clear.to_string(),
            salt: user.pwd_salt.to_string(),
        })?;

        sqlb::update()
            .table(Self::TABLE)
            .and_where("id", "=", id)
            .data(vec![("pwd", pwd.to_string()).into()])
            .exec(db)
            .await?;

        Ok(())
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
