use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value, Value};

use tracing::debug;

use crate::{ctx::CTX, model::ModelManager, rpc::task::list_tasks, Error};

use self::model::RpcRequest;

mod model;
mod router;
mod task;

pub async fn rpc_handler(
    State(manager): State<ModelManager>,
    ctx: CTX,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let RpcRequest { id, method, params } = rpc_req;

    debug!("{:12} - rpc handler - method: {method}", "HANDLER");

    let result_json: Value = match method.as_str() {
        "create_task" => todo!(),
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
