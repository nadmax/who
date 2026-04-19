pub mod jwt;

use axum::Router;

pub fn router() -> Router {
    Router::new().nest("/auth", jwt::router())
}
