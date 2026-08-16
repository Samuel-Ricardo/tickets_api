use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{from_value, json, to_value, Value};

use tracing::debug;

use crate::{
    ctx::CTX,
    model::{task::TaskForCreate, ModelManager},
    rpc::{
        model::ParamsForCreate,
        task::{create_task, delete_task, list_tasks, update_task},
    },
    Error,
};

use self::model::RpcRequest;

mod model;
pub mod router;
mod task;

macro_rules! exec_rpc_fn {
    ($rpc_fn:expr, $ctx:expr, $manager:expr) => {
        $rpc_fn($ctx, $manager)
            .await
            .map(to_value)
            .unwrap()
            .unwrap()
    };

    ($rpc_fn:expr, $ctx:expr, $manager:expr, $params:expr) => {{
        let data = $params.ok_or(Error::RpcMissingParams);
        let data = from_value(data.unwrap())
            .map_err(|_| Error::RpcFailJsonParams)
            .unwrap();

        $rpc_fn($ctx, $manager, data)
            .await
            .map(to_value)
            .unwrap()
            .unwrap()
    }};
}

pub async fn rpc_handler(
    State(manager): State<ModelManager>,
    ctx: CTX,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let RpcRequest { id, method, params } = rpc_req;

    debug!("{:12} - rpc handler - method: {method}", "HANDLER");

    let result_json: Value = match method.as_str() {
        "create_task" => exec_rpc_fn!(create_task, ctx, manager, params),
        "list_tasks" => exec_rpc_fn!(list_tasks, ctx, manager),
        "update_task" => exec_rpc_fn!(update_task, ctx, manager, params),
        "delete_task" => exec_rpc_fn!(delete_task, ctx, manager, params),
        _ => return Err(Error::UnkownRpcMethod(method)).unwrap(),
    };

    let body_response = json!({
        "id": id,
        "result": result_json
    });

    Json(body_response).into_response()
}
