use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("missing authorization header")]
    MissingToken,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidToken  => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::MissingToken  => (StatusCode::UNAUTHORIZED, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}