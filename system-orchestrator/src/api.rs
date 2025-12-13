use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::state::SharedSystemState;
use crate::admin_client::AdminClient;
use tower_http::cors::CorsLayer;
use log::{info, error};

// App State to share with routes
#[derive(Clone)]
pub struct AppState {
    pub system_state: SharedSystemState,
    pub admin_client: Arc<AdminClient>,
    pub tx: broadcast::Sender<String>, // Channel for WS updates
}

pub async fn run_api_server(
    system_state: SharedSystemState, 
    admin_client: Arc<AdminClient>,
    tx: broadcast::Sender<String>,
    port: u16
) {
    let app_state = AppState {
        system_state,
        admin_client,
        tx,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/state", get(state_handler))
        .route("/ws", get(ws_handler))
        .route("/admin", post(admin_handler))
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

// State Handler
async fn state_handler(State(state): State<AppState>) -> Json<Value> {
    let read_guard = state.system_state.read().unwrap();
    // Serialize the entire state (SystemView) to JSON
    Json(serde_json::to_value(&*read_guard).unwrap_or(Value::Null))
}

// WebSocket Handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
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
        Ok(response) => {
            // Try to parse response as JSON, otherwise return as string in JSON object
            match serde_json::from_str::<Value>(&response) {
                Ok(v) => Json(v),
                Err(_) => Json(serde_json::json!({"status": "RAW", "msg": response})),
            }
        },
        Err(e) => {
            error!("Admin Command Failed: {}", e);
            Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()}))
        }
    }
}
