pub mod auth;
pub mod health;

use crate::models::health::HealthResponse;
use axum::Router;
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(health::handler, auth::jwt::login, auth::jwt::me),
    components(schemas(HealthResponse))
)]
struct ApiDoc;

pub fn public_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health::handler))
        .merge(auth::router())
}
