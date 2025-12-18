//! API module
//!
//! This module provides the API for the front-end dashboard.

use crate::admin_client::AdminClient;
use crate::layout::manager::LayoutManager;
use crate::layout::models::layout::Layout;
use crate::process::manager::ProcessManager;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log::{error, info};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

// App State to share with routes
#[derive(Clone)]
pub struct AppState {
    pub admin_client: Arc<AdminClient>,
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub layout_manager: Arc<LayoutManager>,
    pub tx: broadcast::Sender<String>,
}

pub async fn run_api_server(
    admin_client: Arc<AdminClient>,
    process_manager: Arc<Mutex<ProcessManager>>,
    layout_manager: Arc<LayoutManager>,
    tx: broadcast::Sender<String>,
    port: u16,
) {
    let app_state = AppState {
        admin_client,
        process_manager,
        layout_manager,
        tx,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws_handler))
        .route("/admin", post(admin_handler))
        .route("/processes", get(process_handler))
        .route(
            "/layout/:layout_id",
            get(get_layout).post(add_layout).delete(remove_layout),
        )
        .route("/layout/:layout_id/deploy", post(deploy_layout))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    info!("API Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

// WebSocket Handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket
            .send(axum::extract::ws::Message::Text(msg))
            .await
            .is_err()
        {
            break;
        }
    }
}

// Admin Handler
async fn admin_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let cmd_str = payload.to_string();
    info!("API Received Admin Command: {}", cmd_str);

    match state.admin_client.send_command(&cmd_str) {
        Ok(response) => match serde_json::from_str::<Value>(&response) {
            Ok(v) => Json(v),
            Err(_) => Json(serde_json::json!({"status": "RAW", "msg": response})),
        },
        Err(e) => {
            error!("Admin Command Failed: {}", e);
            Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()}))
        }
    }
}

// Process handler
async fn process_handler(State(state): State<AppState>) -> impl IntoResponse {
    let lst = state.process_manager.lock().unwrap().list();

    Json(serde_json::json!({"status": "OK", "processes": lst}))
}

// Layout Handlers

async fn get_layout(
    State(state): State<AppState>,
    axum::extract::Path(layout_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let layout = state.layout_manager.get_layout(&layout_id);
    match layout {
        Some(layout) => Json(serde_json::json!({"status": "OK", "layout": layout})),
        None => Json(serde_json::json!({"status": "ERROR", "msg": "Layout not found"})),
    }
}

async fn add_layout(
    State(state): State<AppState>,
    axum::extract::Path(layout_id): axum::extract::Path<String>,
    Json(layout): Json<Layout>,
) -> impl IntoResponse {
    // We ignore error handling for save in this simple responder
    let _ = state.layout_manager.save_layout(layout_id, layout);
    Json(serde_json::json!({"status": "OK", "msg": "Layout saved"}))
}

async fn remove_layout(
    State(state): State<AppState>,
    axum::extract::Path(layout_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = state.layout_manager.remove_layout(&layout_id);
    Json(serde_json::json!({"status": "OK", "msg": "Layout removed"}))
}

async fn deploy_layout(
    State(state): State<AppState>,
    axum::extract::Path(layout_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let layout = state.layout_manager.get_layout(&layout_id);
    match layout {
        Some(layout) => {
            info!("Deploying layout: {}", layout_id);
            let mut pm = state.process_manager.lock().unwrap();
            match pm.deploy(&layout) {
                Ok(_) => Json(serde_json::json!({"status": "OK", "msg": "Layout deployed"})),
                Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
            }
        }
        None => Json(serde_json::json!({"status": "ERROR", "msg": "Layout not found"})),
    }
}
