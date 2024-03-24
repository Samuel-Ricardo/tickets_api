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
        task::{create_task, list_tasks},
    },
    Error,
};

use self::model::RpcRequest;

mod model;
pub mod router;
mod task;

pub async fn rpc_handler(
    State(manager): State<ModelManager>,
    ctx: CTX,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let RpcRequest { id, method, params } = rpc_req;

    debug!("{:12} - rpc handler - method: {method}", "HANDLER");

    let result_json: Value = match method.as_str() {
        "create_task" => {
            let data = params.ok_or(Error::RpcMissingParams);
            let data = from_value(data.unwrap())
                .map_err(|_| Error::RpcFailJsonParams)
                .unwrap();

            create_task(ctx, manager, ParamsForCreate { data })
                .await
                .map(to_value)
                .unwrap()
                .unwrap()
        }
        "list_tasks" => list_tasks(ctx, manager)
            .await
            .map(to_value)
            .unwrap()
            .unwrap(),
        "update_task" => todo!(),
        "delete_task" => todo!(),
        _ => return Err(Error::UnkownRpcMethod(method)).unwrap(),
    };

    let body_response = json!({
        "id": id,
        "result": result_json
    });

    Json(body_response).into_response()
}
