use crate::model::task::{Task, TaskForCreate, TaskForUpdate};
use crate::model::ModelManager;
use crate::service::task::TaskService;
use crate::{ctx::CTX, Result};

use super::model::{ParamsForCreate, ParamsForUpdate, ParamsIded};

pub async fn create_task(
    ctx: CTX,
    manager: ModelManager,
    params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {
    let ParamsForCreate { data } = params;

    let id = TaskService::create(&ctx, &manager, data).await.unwrap();
    let task = TaskService::get(&ctx, &manager, id).await.unwrap();

    Ok(task)
}

pub async fn list_tasks(ctx: CTX, manager: ModelManager) -> Result<Vec<Task>> {
    let tasks = TaskService::list(&ctx, &manager).await.unwrap();
    Ok(tasks)
}

pub async fn update_task(
    ctx: CTX,
    manager: ModelManager,
    params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {
    let ParamsForUpdate { id, data } = params;
    TaskService::update(&ctx, &manager, id, data).await.unwrap();

    let task = TaskService::get(&ctx, &manager, id).await.unwrap();

    Ok(task)
}

pub async fn delete_task(ctx: CTX, manager: ModelManager, params: ParamsIded) -> Result<Task> {
    let task = TaskService::get(&ctx, &manager, params.id).await.unwrap();
    TaskService::delete(&ctx, &manager, params.id)
        .await
        .unwrap();

    Ok(task)
}
