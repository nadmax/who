use crate::models::hello::HelloResponse;
use axum::Json;

pub async fn handler() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello, World!".to_string(),
    })
}
