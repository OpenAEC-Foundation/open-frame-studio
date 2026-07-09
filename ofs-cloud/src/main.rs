use axum::{routing::{get, post, delete}, Router, Json};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod api;
mod storage;

#[tokio::main]
async fn main() {
    let state = Arc::new(storage::CloudState::new());

    let app = Router::new()
        .route("/api/v1/health", get(|| async { Json(serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")})) }))
        .route("/api/v1/projects", get(api::list_projects))
        .route("/api/v1/projects", post(api::create_project))
        .route("/api/v1/projects/{id}", get(api::get_project))
        .route("/api/v1/projects/{id}", delete(api::delete_project))
        .route("/api/v1/projects/{id}/kozijnen", get(api::get_kozijnen))
        .route("/api/v1/projects/{id}/export/ifc", post(api::export_ifc))
        .route("/api/v1/projects/{id}/quotation", get(api::get_quotation))
        .route("/api/v1/projects/{id}/production", get(api::get_production))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("Open Frame Studio Cloud API running on http://localhost:3456");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3456").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
