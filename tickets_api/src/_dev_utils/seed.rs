use crate::{
    ctx::CTX,
    model::{
        self,
        task::{Task, TaskForCreate},
        ModelManager,
    },
    service::task::TaskService,
};

pub async fn task(
    ctx: &CTX,
    manager: &ModelManager,
    titles: &[&str],
) -> model::error::Result<Vec<Task>> {
    let mut tasks = Vec::new();

    for title in titles {
        let id = TaskService::create(
            ctx,
            manager,
            TaskForCreate {
                title: title.to_string(),
            },
        )
        .await?;

        let task = TaskService::get(ctx, manager, id).await?;

        tasks.push(task);
    }

    Ok(tasks)
}
