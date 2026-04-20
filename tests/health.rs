use axum_test::TestServer;
use who::{models::health::HealthResponse, routes};

fn test_server() -> TestServer {
    let app = routes::public_router();
    TestServer::new(app)
}

#[tokio::test]
async fn test_health() {
    let server = test_server();

    let response = server
        .get("/health")
        .await;

    response.assert_status_ok();
    let body = response.json::<HealthResponse>();
    assert_eq!(body.status, "ok");
}