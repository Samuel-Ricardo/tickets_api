use axum::{http::StatusCode, response::Response};

pub async fn hello_handler() -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .body("{\"hello\":\"world\"}".to_string())
        .unwrap_or_default()
}
