use axum_test::TestServer;
use serde_json::json;
use who::routes;

fn test_server() -> TestServer {
    let app = routes::auth_router();
    TestServer::new(app)
}

#[tokio::test]
async fn test_login_valid() {
    let server = test_server();

    let response = server
        .post("/auth/jwt/login")
        .json(&json!({ "user_id": "123", "role": "admin" }))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert!(body["token"].is_string());
}

#[tokio::test]
async fn test_login_missing_fields() {
    let server = test_server();

    let response = server
        .post("/auth/jwt/login")
        .json(&json!({ "user_id": "123" }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_me_valid_token() {
    let server = test_server();

    let login = server
        .post("/auth/jwt/login")
        .json(&json!({ "user_id": "123", "role": "admin" }))
        .await;

    let token = login.json::<serde_json::Value>()["token"]
        .as_str()
        .unwrap()
        .to_string();

    let response = server
        .get("/auth/jwt/me")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["user_id"], "123");
    assert_eq!(body["role"], "admin");
}

#[tokio::test]
async fn test_me_missing_header() {
    let server = test_server();

    let response = server.get("/auth/jwt/me").await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_me_invalid_token() {
    let server = test_server();

    let response = server
        .get("/auth/jwt/me")
        .add_header("Authorization", "Bearer invalidtoken")
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_me_expired_token() {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use who::routes::auth::jwt::Claims;

    let claims = Claims {
        sub: "123".to_string(),
        role: "admin".to_string(),
        exp: 0,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(b"changeme"),
    )
    .unwrap();

    let server = test_server();
    let response = server
        .get("/auth/jwt/me")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    response.assert_status_unauthorized();
}
