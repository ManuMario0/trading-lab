mod layout_manager;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use layout_manager::LayoutManager;
use log::info;
use orchestrator_protocol::{
    Layout, OrchestratorClient, OrchestratorCommand, OrchestratorResponse, RunMode,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use trading_core::comms::transports::zmq::ZmqClientDuplex;

// App State
#[derive(Clone)]
struct AppState {
    client: Arc<Mutex<OrchestratorClient>>,
    layout_manager: Arc<LayoutManager>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("=== Supervisor Backend Starting ===");

    // 1. Initialize Layout Manager
    let layout_manager = Arc::new(LayoutManager::new());

    // 2. Initialize Orchestrator Client
    let addr = "tcp://127.0.0.1:5555";
    info!("Connecting to Orchestrator Daemon at {}", addr);
    let transport = Box::new(ZmqClientDuplex::new(addr)?);
    let client = OrchestratorClient::new(transport);
    let client = Arc::new(Mutex::new(client));

    let state = AppState {
        client,
        layout_manager,
    };

    // 3. Setup Routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/processes", get(get_processes))
        .route(
            "/layout/:layout_id",
            get(get_layout).post(save_layout).delete(remove_layout),
        )
        .route("/layout/:layout_id/deploy", post(deploy_layout)) // Uses REST for deploy now, simpler than socket?
        // Original used /ws for events?
        // Original: ws.on_upgrade(|socket| handle_socket(socket, state))
        // And originally API server subscribed to broadcast tx.
        // Daemon doesn't broadcast events yet. It only replies to commands.
        // So WS is not very useful unless we poll status and push it.
        // For MVP, lets keep REST or simple polling.
        // I will keep WS endpoint stub if frontend expects it, or just use REST.
        // Frontend uses WS for logging updates?
        // Original system-orchestrator broadcasted something?
        // "let (tx, _rx) = broadcast::channel(100);" and passed tx to run_api_server.
        // But nothing was sending to tx in the original code main loop!
        // So usage of WS was likely placeholder.
        // I will omit WS for now as Daemon is request-response.
        .route("/layout/:layout_id/wallet", get(get_wallet))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);
    info!("Supervisor Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_processes(State(state): State<AppState>) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client.send_command(OrchestratorCommand::GetStatus).await {
        Ok(OrchestratorResponse::StatusInfo(info)) => {
            Json(serde_json::json!({"status": "OK", "processes": info}))
        }
        Ok(other) => Json(
            serde_json::json!({"status": "ERROR", "msg": format!("Unexpected response: {:?}", other)}),
        ),
        Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
    }
}

async fn get_layout(
    State(state): State<AppState>,
    Path(layout_id): Path<String>,
) -> impl IntoResponse {
    let layout = state.layout_manager.get_layout(&layout_id);
    match layout {
        Some(layout) => Json(serde_json::json!({"status": "OK", "layout": layout})),
        None => Json(serde_json::json!({"status": "ERROR", "msg": "Layout not found"})),
    }
}

async fn save_layout(
    State(state): State<AppState>,
    Path(layout_id): Path<String>,
    Json(layout): Json<Layout>,
) -> impl IntoResponse {
    match state.layout_manager.save_layout(layout_id, layout) {
        Ok(_) => Json(serde_json::json!({"status": "OK", "msg": "Layout saved"})),
        Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
    }
}

async fn remove_layout(
    State(state): State<AppState>,
    Path(layout_id): Path<String>,
) -> impl IntoResponse {
    match state.layout_manager.remove_layout(&layout_id) {
        Ok(_) => Json(serde_json::json!({"status": "OK", "msg": "Layout removed"})),
        Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
    }
}

async fn deploy_layout(
    State(state): State<AppState>,
    Path(layout_id): Path<String>,
) -> impl IntoResponse {
    // 1. Get layout
    let layout = match state.layout_manager.get_layout(&layout_id) {
        Some(l) => l,
        None => return Json(serde_json::json!({"status": "ERROR", "msg": "Layout not found"})),
    };

    // 2. Send Deploy Command
    let mut client = state.client.lock().await;
    let mode = RunMode::Live; // Default or allow param?

    match client
        .send_command(OrchestratorCommand::Deploy { layout, mode })
        .await
    {
        Ok(OrchestratorResponse::Success(msg)) => {
            Json(serde_json::json!({"status": "OK", "msg": msg}))
        }
        Ok(OrchestratorResponse::Error(e)) => {
            Json(serde_json::json!({"status": "ERROR", "msg": e}))
        }
        Ok(other) => Json(
            serde_json::json!({"status": "ERROR", "msg": format!("Unexpected response: {:?}", other)}),
        ),
        Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
    }
}

async fn get_wallet(
    State(state): State<AppState>,
    Path(layout_id): Path<String>,
) -> impl IntoResponse {
    let mut client = state.client.lock().await;
    match client
        .send_command(OrchestratorCommand::GetWallet { layout_id })
        .await
    {
        Ok(OrchestratorResponse::WalletInfo(val)) => {
            Json(serde_json::json!({"status": "OK", "wallet": val}))
        }
        Ok(OrchestratorResponse::Error(e)) => {
            Json(serde_json::json!({"status": "ERROR", "msg": e}))
        }
        Ok(other) => Json(
            serde_json::json!({"status": "ERROR", "msg": format!("Unexpected response: {:?}", other)}),
        ),
        Err(e) => Json(serde_json::json!({"status": "ERROR", "msg": e.to_string()})),
    }
}
