use axum::{
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod api;
mod service;
mod storage;

#[tokio::main]
async fn main() {
    let service_mode = match std::env::args().nth(1).as_deref() {
        Some("--service") => true,
        None => false,
        Some(_) => {
            eprintln!("Usage: ofs-cloud [--service]");
            std::process::exit(2)
        }
    };
    let app = if service_mode {
        let token = std::env::var("OFS_API_TOKEN").unwrap_or_default();
        service::router(&token).unwrap_or_else(|message| {
            eprintln!("{message}");
            std::process::exit(2)
        })
    } else {
        project_prototype()
    };
    let address = std::env::var("OFS_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3456".into());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("bind API address");
    println!(
        "Open Frame Studio Cloud API listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

fn project_prototype() -> Router {
    let state = Arc::new(storage::CloudState::new());

    Router::new()
        .route(
            "/api/v1/health",
            get(|| async {
                Json(serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
            }),
        )
        .route("/api/v1/projects", get(api::list_projects))
        .route("/api/v1/projects", post(api::create_project))
        .route("/api/v1/projects/{id}", get(api::get_project))
        .route("/api/v1/projects/{id}", delete(api::delete_project))
        .route("/api/v1/projects/{id}/kozijnen", get(api::get_kozijnen))
        .route("/api/v1/projects/{id}/export/ifc", post(api::export_ifc))
        .route("/api/v1/projects/{id}/quotation", get(api::get_quotation))
        .route("/api/v1/projects/{id}/production", get(api::get_production))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
