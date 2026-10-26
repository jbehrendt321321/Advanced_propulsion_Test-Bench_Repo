use std::fs::{create_dir_all, OpenOptions};
use std::path::Path;
use tokio::sync::watch;
use tracing::{error, info};

use crate::state::BenchTelemetry;

pub struct CsvLogger {
    log_dir: String,
}

impl CsvLogger {
    pub fn new(log_dir: &str) -> Self {
        Self {
            log_dir: log_dir.to_string(),
        }
    }

    pub async fn run(self, mut state_rx: watch::Receiver<BenchTelemetry>) {
        if let Err(e) = create_dir_all(&self.log_dir) {
            error!("Failed to create log directory: {e}");
            return;
        }

        let now = chrono::Local::now();
        let filename = format!("{}/testrun_{}.csv", self.log_dir, now.format("%Y%m%d_%H%M%S"));
        info!("Writing unified telemetry CSV to: {}", filename);

        let file = match OpenOptions::new().create(true).write(true).truncate(true).open(&filename) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open log file {}: {e}", filename);
                return;
            }
        };

        let mut wtr = csv::WriterBuilder::new().from_writer(file);

        // Header row
        let _ = wtr.write_record(&[
            "timestamp_ms",
            "pump_power",
            "pump_out_vlv",
            "oxidizer_circ_vlv",
            "fuel_circ_vlv",
            "film_circ_vlv",
            "pump_out_pres_bar",
            "fuel_circ_pres_bar",
            "oxidizer_circ_pres_bar",
            "film_circ_pres_bar",
            "fuel_circ_mass_lpm",
            "oxidizer_circ_mass_lpm",
            "film_circ_mass_lpm",
        ]);
        let _ = wtr.flush();

        while state_rx.changed().await.is_ok() {
            let t = state_rx.borrow().clone();

            let row = [
                t.timestamp_ms.to_string(),
                t.actuators.pump_power.to_string(),
                t.actuators.pump_out_vlv.to_string(),
                t.actuators.oxidizer_circ_vlv.to_string(),
                t.actuators.fuel_circ_vlv.to_string(),
                t.actuators.film_circ_vlv.to_string(),
                format!("{:.3}", t.pressures.pump_out_pres),
                format!("{:.3}", t.pressures.fuel_circ_pres),
                format!("{:.3}", t.pressures.oxidizer_circ_pres),
                format!("{:.3}", t.pressures.film_circ_pres),
                format!("{:.2}", t.flows.fuel_circ_mass),
                format!("{:.2}", t.flows.oxidizer_circ_mass),
                format!("{:.2}", t.flows.film_circ_mass),
            ];

            if wtr.write_record(&row).is_err() {
                error!("Error appending CSV record");
                break;
            }
            let _ = wtr.flush();
        }
    }
}