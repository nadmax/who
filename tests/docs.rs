use axum::http::StatusCode;
use axum_test::TestServer;
use who::routes;

fn test_server() -> TestServer {
    let app = routes::public_router();
    TestServer::new(app)
}

#[tokio::test]
async fn test_swagger_ui_redirect() {
    let server = test_server();
    let response = server.get("/").await;

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_openapi_json() {
    let server = test_server();
    let response = server.get("/api-docs/openapi.json").await;
    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"].is_object());
}