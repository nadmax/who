pub mod models;
pub mod routes;

use axum::Router;

pub fn app() -> Router {
    Router::new()
        .merge(routes::public_router())
        .merge(routes::auth_router())
}
