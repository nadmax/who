pub mod models;
pub mod routes;
pub mod services;

use axum::Router;

pub fn app() -> Router {
    Router::new().merge(routes::public_router())
}
