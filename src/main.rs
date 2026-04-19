mod models;
mod routes;

use axum::{Router, routing::get};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(routes::hello::handler))
        .route("/health", get(routes::health::handler));

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://localhost:8080");
    axum::serve(listener, app).await?;

    Ok(())
}
