use crate::ctx::CTX;
use crate::model::error::Result;
use crate::model::task::{Task, TaskForCreate};
use crate::model::ModelManager;

pub struct TaskService;

impl TaskService {
    pub async fn create(_ctx: &CTX, manager: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        let db = manager.db();

        let (id,) =
            sqlx::query_as::<_, (i64,)>("INSERT INTO tasks (title) VALUES ($1) RETURNING id")
                .bind(task_c.title)
                .fetch_one(db)
                .await?;

        Ok(id)
    }

    pub async fn get(_ctx: &CTX, manager: &ModelManager, id: i64) -> Result<Task> {
        let db = manager.db();

        let task: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(crate::model::error::Error::EntityNotFound { entity: "task", id })?;

        Ok(task)
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
}

// INFO: END REGION [tests]
