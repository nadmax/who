pub mod auth;
pub mod health;
pub mod hello;

use axum::{Router, routing::get};

pub fn public_router() -> Router {
    Router::new()
        .route("/", get(hello::handler))
        .route("/health", get(health::handler))
}

pub fn auth_router() -> Router {
    auth::router()
}
