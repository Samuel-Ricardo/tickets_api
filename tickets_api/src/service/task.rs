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
        let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
            .bind(id)
            .fetch_one(manager.db())
            .await?;

        assert_eq!(title, fixtures_title);

        // INFO: delete the created task
        let count = sqlx::query("DELETE FROM task WHERE id = $1")
            .bind(id)
            .execute(manager.db())
            .await?
            .rows_affected();

        assert_eq!(count, 1);

        Ok(())
    }
}

// INFO: END REGION [tests]
