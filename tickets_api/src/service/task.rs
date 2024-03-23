use crate::base;
use crate::base::db::DbBmc;
use crate::ctx::CTX;
use crate::model::error::Result;
use crate::model::task::{Task, TaskForCreate};
use crate::model::ModelManager;

pub struct TaskService;

impl DbBmc for TaskService {
    const TABLE: &'static str = "tasks";
}

impl TaskService {
    pub async fn create(ctx: &CTX, manager: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        base::db::create::<Self, _>(ctx, manager, task_c).await
    }

    pub async fn get(ctx: &CTX, manager: &ModelManager, id: i64) -> Result<Task> {
        base::db::get::<Self, Task>(ctx, manager, id).await
    }

    pub async fn list(ctx: &CTX, manager: &ModelManager) -> Result<Vec<Task>> {
        base::db::list::<Self, Task>(ctx, manager).await
    }

    pub async fn delete(_ctx: &CTX, manager: &ModelManager, id: i64) -> Result<()> {
        let db = manager.db();

        let count = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

        if count == 0 {
            return Err(crate::model::error::Error::EntityNotFound { entity: "task", id });
        }

        Ok(())
    }
}

// INFO: REGION [tests]

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use serial_test::serial;

    use super::*;
    use crate::_dev_utils;

    #[serial]
    #[tokio::test]
    async fn test_create() -> Result<()> {
        let manager = _dev_utils::init_test_db().await;
        let ctx = CTX::root_ctx();
        let fixtures_title: &str = "test_create_ok title";

        let task_c: TaskForCreate = TaskForCreate {
            title: fixtures_title.to_string(),
        };

        let id = TaskService::create(&ctx, &manager, task_c).await?;

        // INFO: check the created task
        let task = TaskService::get(&ctx, &manager, id).await?;

        assert_eq!(task.title, fixtures_title);

        // INFO: delete the created task
        TaskService::delete(&ctx, &manager, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list() -> Result<()> {
        let manager = _dev_utils::init_test_db().await;
        let ctx = CTX::root_ctx();
        const titles: &[&str] = &["test_list_1", "test_list_2", "test_list_3"];

        _dev_utils::seed::task(&ctx, &manager, titles).await?;

        let tasks = TaskService::list(&ctx, &manager).await?;

        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| titles.contains(&t.title.as_str()))
            .collect();

        assert_eq!(tasks.len(), titles.len(), "tasks should be equal");
        assert!(!tasks.is_empty(), "tasks should not be empty");

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        let manager = _dev_utils::init_test_db().await;
        let ctx = CTX::root_ctx();
        let fx_id = 100;

        let res = TaskService::get(&ctx, &manager, fx_id).await;

        assert!(
            matches!(
                res,
                Err(crate::model::error::Error::EntityNotFound { entity, id })
            ),
            "EntityNotFound not Match",
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        let manager = _dev_utils::init_test_db().await;
        let ctx = CTX::root_ctx();
        let fx_id = 100;

        let res = TaskService::delete(&ctx, &manager, fx_id).await;

        assert!(
            matches!(
                res,
                Err(crate::model::error::Error::EntityNotFound { entity, id })
            ),
            "EntityNotFound not Match",
        );

        Ok(())
    }
}

// INFO: END REGION [tests]
