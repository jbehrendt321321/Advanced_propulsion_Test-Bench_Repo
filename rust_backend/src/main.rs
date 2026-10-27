use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};
use tokio_modbus::prelude::*;
use tokio_serial::SerialPort;
use tower_http::cors::CorsLayer;

const COIL_BASE: u16 = 0x100;
const REG_BASE: u16 = 0x300;

#[derive(Debug, Clone, Serialize, Default)]
pub struct BenchState {
    pub online: bool,
    // Actuators
    pub pump_power: bool,
    pub pump_out_vlv: bool,
    pub oxidizer_circ_vlv: bool,
    pub fuel_circ_vlv: bool,
    pub film_circ_vlv: bool,
    // Pressures (bar)
    pub pump_out_pres: f32,
    pub fuel_circ_pres: f32,
    pub oxidizer_circ_pres: f32,
    pub film_circ_pres: f32,
    // Mass Flows (L/min)
    pub fuel_circ_mass: f32,
    pub oxidizer_circ_mass: f32,
    pub film_circ_mass: f32,
}

#[derive(Debug, Deserialize)]
pub struct ActuatorCommand {
    pub name: String,
    pub value: bool,
}

pub struct AppState {
    pub bench: RwLock<BenchState>,
    pub cmd_tx: mpsc::Sender<ActuatorCommand>,
}

fn decode_f32(hi: u16, lo: u16) -> f32 {
    f32::from_bits(u32::from(lo) | (u32::from(hi) << 16))
}

#[tokio::main]
async fn main() {
    let serial_port = std::env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/ttyUSB0".to_string());
    let mock_mode = std::env::var("MOCK_PLC").is_ok();

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ActuatorCommand>(32);
    let app_state = Arc::new(AppState {
        bench: RwLock::new(BenchState::default()),
        cmd_tx,
    });

    // Unified CSV log file
    let mut csv_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("coldflow_log.csv")
        .expect("Failed to open coldflow_log.csv");
    writeln!(
        csv_file,
        "time_s,pump_power,pump_out_vlv,ox_circ_vlv,fuel_circ_vlv,film_circ_vlv,\
         pump_out_pres_bar,fuel_circ_pres_bar,ox_circ_pres_bar,film_circ_pres_bar,\
         fuel_circ_mass_lmin,ox_circ_mass_lmin,film_circ_mass_lmin"
    )
    .ok();
    let start_time = Instant::now();

    // 20 Hz Polling & Command Worker
    let worker_state = app_state.clone();
    tokio::spawn(async move {
        let mut ctx_opt: Option<client::Context> = None;

        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            // --- Mock Mode ---
            if mock_mode {
                let mut s = worker_state.bench.write().await;
                s.online = true;
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd.name.as_str() {
                        "pump_power" => s.pump_power = cmd.value,
                        "pump_out_vlv" => s.pump_out_vlv = cmd.value,
                        "oxidizer_circ_vlv" => s.oxidizer_circ_vlv = cmd.value,
                        "fuel_circ_vlv" => s.fuel_circ_vlv = cmd.value,
                        "film_circ_vlv" => s.film_circ_vlv = cmd.value,
                        _ => {}
                    }
                }

                let pwr = s.pump_power;
                s.pump_out_pres = if pwr { 16.5 } else { 0.1 };
                s.fuel_circ_pres = if pwr && s.fuel_circ_vlv { 14.2 } else { 0.05 };
                s.oxidizer_circ_pres = if pwr && s.oxidizer_circ_vlv { 14.5 } else { 0.05 };
                s.film_circ_pres = if pwr && s.film_circ_vlv { 12.8 } else { 0.02 };

                s.fuel_circ_mass = if pwr && s.fuel_circ_vlv { 18.4 } else { 0.0 };
                s.oxidizer_circ_mass = if pwr && s.oxidizer_circ_vlv { 21.0 } else { 0.0 };
                s.film_circ_mass = if pwr && s.film_circ_vlv { 6.2 } else { 0.0 };

                let t = start_time.elapsed().as_secs_f32();
                let _ = writeln!(
                    csv_file,
                    "{:.3},{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
                    t,
                    s.pump_power as u8,
                    s.pump_out_vlv as u8,
                    s.oxidizer_circ_vlv as u8,
                    s.fuel_circ_vlv as u8,
                    s.film_circ_vlv as u8,
                    s.pump_out_pres,
                    s.fuel_circ_pres,
                    s.oxidizer_circ_pres,
                    s.film_circ_pres,
                    s.fuel_circ_mass,
                    s.oxidizer_circ_mass,
                    s.film_circ_mass
                );
                continue;
            }

            // --- Hardware Connection ---
            if ctx_opt.is_none() {
                let builder = tokio_serial::new(&serial_port, 115200);
                match tokio_serial::SerialStream::open(&builder) {
                    Ok(mut port) => {
                        let _ = port.write_data_terminal_ready(false);
                        let ctx = rtu::attach_slave(port, Slave(1));
                        println!("Connected to Controllino on {}", serial_port);
                        ctx_opt = Some(ctx);
                    }
                    Err(_) => {
                        worker_state.bench.write().await.online = false;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }

            let ctx = ctx_opt.as_mut().unwrap();

            // 1. Process queued actuator commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                let coil_addr = match cmd.name.as_str() {
                    "pump_power" => Some(0x100),
                    "pump_out_vlv" => Some(0x101),
                    "oxidizer_circ_vlv" => Some(0x102),
                    "fuel_circ_vlv" => Some(0x103),
                    "film_circ_vlv" => Some(0x104),
                    _ => None,
                };
                if let Some(addr) = coil_addr {
                    let _ = tokio::time::timeout(
                        Duration::from_millis(200),
                        ctx.write_single_coil(addr, cmd.value),
                    )
                    .await;
                }
            }

            // 2. Poll Coils (0x100..0x104) and Holding Registers (0x300..0x30D)
            let coil_res = tokio::time::timeout(
                Duration::from_millis(200),
                ctx.read_coils(COIL_BASE, 5),
            )
            .await;

            let reg_res = tokio::time::timeout(
                Duration::from_millis(200),
                ctx.read_holding_registers(REG_BASE, 14),
            )
            .await;

            match (coil_res, reg_res) {
                (Ok(Ok(Ok(coils))), Ok(Ok(Ok(regs))))
                    if coils.len() >= 5 && regs.len() == 14 =>
                {
                    let mut s = worker_state.bench.write().await;
                    s.online = true;

                    // Coils
                    s.pump_power = coils[0];
                    s.pump_out_vlv = coils[1];
                    s.oxidizer_circ_vlv = coils[2];
                    s.fuel_circ_vlv = coils[3];
                    s.film_circ_vlv = coils[4];

                    // Pressures (0x300..0x307)
                    s.pump_out_pres = decode_f32(regs[0], regs[1]);
                    s.fuel_circ_pres = decode_f32(regs[2], regs[3]);
                    s.oxidizer_circ_pres = decode_f32(regs[4], regs[5]);
                    s.film_circ_pres = decode_f32(regs[6], regs[7]);

                    // Mass flows (0x308..0x30D)
                    s.fuel_circ_mass = decode_f32(regs[8], regs[9]);
                    s.oxidizer_circ_mass = decode_f32(regs[10], regs[11]);
                    s.film_circ_mass = decode_f32(regs[12], regs[13]);

                    let t = start_time.elapsed().as_secs_f32();
                    let _ = writeln!(
                        csv_file,
                        "{:.3},{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
                        t,
                        s.pump_power as u8,
                        s.pump_out_vlv as u8,
                        s.oxidizer_circ_vlv as u8,
                        s.fuel_circ_vlv as u8,
                        s.film_circ_vlv as u8,
                        s.pump_out_pres,
                        s.fuel_circ_pres,
                        s.oxidizer_circ_pres,
                        s.film_circ_pres,
                        s.fuel_circ_mass,
                        s.oxidizer_circ_mass,
                        s.film_circ_mass
                    );
                }
                _ => {
                    eprintln!("Modbus timeout/error, resetting connection...");
                    worker_state.bench.write().await.online = false;
                    ctx_opt = None;
                }
            }
        }
    });

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/actuator", post(set_actuator))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    println!("Backend running on http://localhost:3001");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_state(State(state): State<Arc<AppState>>) -> Json<BenchState> {
    Json(state.bench.read().await.clone())
}

async fn set_actuator(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<ActuatorCommand>,
) -> Json<&'static str> {
    let _ = state.cmd_tx.try_send(cmd);
    Json("ok")
}