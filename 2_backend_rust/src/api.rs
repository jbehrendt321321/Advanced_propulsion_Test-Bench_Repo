use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::state::{ActuatorCommand, BenchTelemetry};

#[derive(Clone)]
pub struct AppState {
    pub state_rx: watch::Receiver<BenchTelemetry>,
    pub cmd_tx: mpsc::Sender<ActuatorCommand>,
}

pub fn create_router(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(cors)
        .with_state(app_state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.state_rx.clone();
    let cmd_tx = state.cmd_tx.clone();

    // Spawn outbound telemetry streaming task (at 20 Hz / state changes)
    let send_task = tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let current = rx.borrow().clone();
            if let Ok(json) = serde_json::to_string(&current) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Inbound command handling task
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ActuatorCommand>(&text) {
                    Ok(cmd) => {
                        info!("Received actuator command from UI: {:?}", cmd);
                        if let Err(e) = cmd_tx.send(cmd).await {
                            error!("Failed to forward UI command to driver: {e}");
                        }
                    }
                    Err(e) => {
                        warn!("Unrecognized message received from client: {} ({e})", text);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}