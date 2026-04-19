use crate::models::health::HealthResponse;
use axum::Json;

pub async fn handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
