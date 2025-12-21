use axum::{
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use crate::admin::registry::GLOBAL_REGISTRY;
use serde_json::json;

pub async fn start_admin_server(port: u16) {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/schema", get(get_schema))
        .route("/update", post(update_param));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Admin server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_schema() -> Json<serde_json::Value> {
    let registry = GLOBAL_REGISTRY.lock().unwrap();
    let params = registry.get_all();
    Json(json!(params))
}

#[derive(serde::Deserialize)]
struct UpdateRequest {
    name: String,
    value: String,
}

async fn update_param(Json(payload): Json<UpdateRequest>) -> Json<serde_json::Value> {
    let mut registry = GLOBAL_REGISTRY.lock().unwrap();
    match registry.update(&payload.name, &payload.value) {
        Ok(_) => Json(json!({"status": "ok", "message": "Updated"})),
        Err(e) => Json(json!({"status": "error", "message": e})),
    }
}
