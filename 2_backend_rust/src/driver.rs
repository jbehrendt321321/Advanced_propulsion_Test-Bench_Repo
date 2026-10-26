use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio_modbus::client::{Context, Reader, Writer};
use tokio_serial::SerialStream;
use tracing::{error, info, warn};

use crate::modbus_map::{coils, holding_registers};
use crate::state::{ActuatorCommand, BenchTelemetry};

pub struct ModbusDriver {
    pub port_path: String,
    pub baud_rate: u32,
    pub poll_interval: Duration,
}

impl ModbusDriver {
    pub fn new(port_path: String, baud_rate: u32, poll_hz: u64) -> Self {
        Self {
            port_path,
            baud_rate,
            poll_interval: Duration::from_millis(1000 / poll_hz),
        }
    }

    pub async fn run(
        self,
        state_tx: watch::Sender<BenchTelemetry>,
        mut cmd_rx: mpsc::Receiver<ActuatorCommand>,
    ) {
        info!("Starting Modbus driver on {}", self.port_path);

        loop {
            match self.connect_and_poll(&state_tx, &mut cmd_rx).await {
                Ok(_) => warn!("Driver exited cleanly. Reconnecting..."),
                Err(err) => {
                    warn!("Modbus connection error on {}: {}. Entering simulation fallback for 5s...", self.port_path, err);
                    self.run_simulation_step(&state_tx, &mut cmd_rx, Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_and_poll(
        &self,
        state_tx: &watch::Sender<BenchTelemetry>,
        cmd_rx: &mut mpsc::Receiver<ActuatorCommand>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let builder = tokio_serial::new(&self.port_path, self.baud_rate)
            .timeout(Duration::from_millis(100));
        let stream = SerialStream::open_by_builders(&builder)?;
        let mut ctx: Context = tokio_modbus::client::rtu::attach_slave(stream, tokio_modbus::Slave(1));

        info!("Modbus RTU connected successfully to Controllino Mini");

        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            interval.tick().await;

            // 1. Process any pending actuator commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                let coil_addr = cmd.actuator.to_coil_address();
                info!("Writing coil 0x{:04X} -> {}", coil_addr, cmd.state);
                ctx.write_single_coil(coil_addr, cmd.state).await?;
            }

            // 2. Bulk read coils
            let coils_res = ctx.read_coils(coils::BASE, coils::COUNT).await?;

            // 3. Bulk read holding registers (7 sensors = 14 registers)
            let regs_res = ctx.read_holding_registers(holding_registers::BASE, holding_registers::COUNT).await?;

            // 4. Update the shared state and broadcast to UI & Logger
            let now_ms = chrono::Utc::now().timestamp_millis() as u64;
            state_tx.send_modify(|state| {
                state.update_from_raw(now_ms, &coils_res, &regs_res);
            });
        }
    }

    /// Simulation fallback when hardware is disconnected
    async fn run_simulation_step(
        &self,
        state_tx: &watch::Sender<BenchTelemetry>,
        cmd_rx: &mut mpsc::Receiver<ActuatorCommand>,
        duration: Duration,
    ) {
        let start = tokio::time::Instant::now();
        let mut sim_interval = tokio::time::interval(self.poll_interval);
        let mut sim_p_pump: f32 = 0.0;

        while start.elapsed() < duration {
            sim_interval.tick().await;

            // Handle commands during simulation
            while let Ok(cmd) = cmd_rx.try_recv() {
                state_tx.send_modify(|s| match cmd.actuator {
                    crate::state::ActuatorId::PumpPower => s.actuators.pump_power = cmd.state,
                    crate::state::ActuatorId::PumpOutVlv => s.actuators.pump_out_vlv = cmd.state,
                    crate::state::ActuatorId::OxidizerCircVlv => s.actuators.oxidizer_circ_vlv = cmd.state,
                    crate::state::ActuatorId::FuelCircVlv => s.actuators.fuel_circ_vlv = cmd.state,
                    crate::state::ActuatorId::FilmCircVlv => s.actuators.film_circ_vlv = cmd.state,
                });
            }

            // Simple physics simulation
            let now_ms = chrono::Utc::now().timestamp_millis() as u64;
            state_tx.send_modify(|s| {
                s.timestamp_ms = now_ms;
                s.connected = false; // Flagged as simulated / hardware offline

                if s.actuators.pump_power {
                    sim_p_pump = (sim_p_pump + 0.8).min(20.5);
                } else {
                    sim_p_pump = (sim_p_pump - 0.5).max(0.0);
                }
                s.pressures.pump_out_pres = sim_p_pump;

                if s.actuators.pump_out_vlv && sim_p_pump > 2.0 {
                    s.pressures.fuel_circ_pres = if s.actuators.fuel_circ_vlv { sim_p_pump * 0.85 } else { 0.0 };
                    s.pressures.oxidizer_circ_pres = if s.actuators.oxidizer_circ_vlv { sim_p_pump * 0.90 } else { 0.0 };
                    s.pressures.film_circ_pres = if s.actuators.film_circ_vlv { sim_p_pump * 0.70 } else { 0.0 };

                    s.flows.fuel_circ_mass = if s.actuators.fuel_circ_vlv { 12.4 } else { 0.0 };
                    s.flows.oxidizer_circ_mass = if s.actuators.oxidizer_circ_vlv { 18.2 } else { 0.0 };
                    s.flows.film_circ_mass = if s.actuators.film_circ_vlv { 4.1 } else { 0.0 };
                } else {
                    s.pressures.fuel_circ_pres = 0.0;
                    s.pressures.oxidizer_circ_pres = 0.0;
                    s.pressures.film_circ_pres = 0.0;
                    s.flows.fuel_circ_mass = 0.0;
                    s.flows.oxidizer_circ_mass = 0.0;
                    s.flows.film_circ_mass = 0.0;
                }
            });
        }
    }
}