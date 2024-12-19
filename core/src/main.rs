use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 9090));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
