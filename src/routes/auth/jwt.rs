use axum::{Json, Router, http::{HeaderMap, StatusCode}, routing::{get, post}};
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa::ToSchema;
use crate::services::auth::jwt as jwt_service;

#[derive(Debug, Deserialize, ToSchema)]
struct LoginRequest {
    user_id: String,
    role: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/jwt/login", post(login))
        .route("/jwt/me", get(me))
}

#[utoipa::path(
    post,
    path = "/auth/jwt/login",
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Unauthorized"),
    )
)]
async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = jwt_service::create_token(&payload.user_id, &payload.role)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to generate token: {}", e) })),
        ))?;
    Ok(Json(json!({ "token": token })))
}

#[utoipa::path(
    get,
    path = "/auth/jwt/me",
    responses(
        (status = 200, description = "Current user"),
        (status = 401, description = "Unauthorized"),
    )
)]
async fn me(headers: HeaderMap) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = jwt_service::extract_bearer_token(&headers).ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Missing or invalid Authorization header" })),
    ))?;

    let claims = jwt_service::decode_token(&token)
        .map_err(|e| (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": format!("Invalid token: {}", e) })),
        ))?;

    Ok(Json(jwt_service::claims_to_json(&claims)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
        Router,
    };
    use tower::ServiceExt;

    fn app() -> Router {
        router()
    }

    #[tokio::test]
    async fn test_login_returns_token() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jwt/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"user_id":"123","role":"admin"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["token"].as_str().is_some_and(|t| !t.is_empty()));
    }

    #[tokio::test]
    async fn test_login_missing_fields_returns_422() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jwt/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"user_id":"123"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_me_with_valid_token() {
        // First get a token
        let login_response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jwt/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"user_id":"123","role":"admin"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
        let login_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = login_json["token"].as_str().unwrap();

        // Then use it
        let response = app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/jwt/me")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["user_id"], "123");
        assert_eq!(json["role"], "admin");
    }

    #[tokio::test]
    async fn test_me_without_token_returns_401() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/jwt/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_me_with_invalid_token_returns_401() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/jwt/me")
                    .header(header::AUTHORIZATION, "Bearer not.a.valid.token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_me_with_malformed_auth_header_returns_401() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/jwt/me")
                    .header(header::AUTHORIZATION, "Basic somebase64==")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}