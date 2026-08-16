use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamsForCreate<D> {
    pub data: D,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamsForUpdate<D> {
    pub id: i64,
    pub data: D,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParamsIded {
    pub id: i64,
}
