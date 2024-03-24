use serde_json::Value;

pub struct RpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct ParamsForCreate<D> {
    pub data: D,
}

pub struct ParamsForUpdate<D> {
    pub id: i64,
    pub data: D,
}

pub struct ParamsIded {
    pub id: i64,
}
