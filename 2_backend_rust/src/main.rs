mod api;
mod driver;
mod logger;
mod modbus_map;
mod state;

use std::env;
use std::net::SocketAddr;
use tokio::sync::{mpsc, watch};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{create_router, AppState};
use crate::driver::ModbusDriver;
use crate::logger::CsvLogger;
use crate::state::{ActuatorCommand, BenchTelemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Cold-Flow Bench Mission Control Backend");

    // Allow overriding port via env: SERIAL_PORT=COM3 or /dev/ttyUSB0
    let serial_port = env::var("SERIAL_PORT").unwrap_or_else(|_| {
        if cfg!(windows) {
            "COM3".to_string()
        } else {
            "/dev/ttyUSB0".to_string()
        }
    });

    let (state_tx, state_rx) = watch::channel(BenchTelemetry::default());
    let (cmd_tx, cmd_rx) = mpsc::channel::<ActuatorCommand>(32);

    // 1. Spawn Modbus RTU polling loop (20 Hz)
    let driver = ModbusDriver::new(serial_port, 115200, 20);
    tokio::spawn(driver.run(state_tx, cmd_rx));

    // 2. Spawn CSV timeseries logger
    let logger = CsvLogger::new("logs");
    tokio::spawn(logger.run(state_rx.clone()));

    // 3. Start Axum HTTP/WebSocket server on port 3001
    let app_state = AppState {
        state_rx,
        cmd_tx,
    };
    let app = create_router(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("P&ID WebSocket Server listening on ws://localhost:3001/ws");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}