use serde::{Deserialize, Serialize};
use crate::modbus_map::coils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActuatorId {
    PumpPower,
    PumpOutVlv,
    OxidizerCircVlv,
    FuelCircVlv,
    FilmCircVlv,
}

impl ActuatorId {
    pub fn to_coil_address(&self) -> u16 {
        match self {
            ActuatorId::PumpPower => coils::PUMP_POWER,
            ActuatorId::PumpOutVlv => coils::PUMP_OUT_VLV,
            ActuatorId::OxidizerCircVlv => coils::OXIDIZER_CIRC_VLV,
            ActuatorId::FuelCircVlv => coils::FUEL_CIRC_VLV,
            ActuatorId::FilmCircVlv => coils::FILM_CIRC_VLV,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorCommand {
    pub actuator: ActuatorId,
    pub state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActuatorStates {
    pub pump_power: bool,
    pub pump_out_vlv: bool,
    pub oxidizer_circ_vlv: bool,
    pub fuel_circ_vlv: bool,
    pub film_circ_vlv: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pressures {
    pub pump_out_pres: f32,
    pub fuel_circ_pres: f32,
    pub oxidizer_circ_pres: f32,
    pub film_circ_pres: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Flows {
    pub fuel_circ_mass: f32,
    pub oxidizer_circ_mass: f32,
    pub film_circ_mass: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchTelemetry {
    pub timestamp_ms: u64,
    pub connected: bool,
    pub actuators: ActuatorStates,
    pub pressures: Pressures,
    pub flows: Flows,
}

impl BenchTelemetry {
    pub fn update_from_raw(&mut self, timestamp_ms: u64, coils_buf: &[bool], regs_buf: &[u16]) {
        use crate::modbus_map::decode_f32;

        self.timestamp_ms = timestamp_ms;
        self.connected = true;

        if coils_buf.len() >= 5 {
            self.actuators.pump_power = coils_buf[0];
            self.actuators.pump_out_vlv = coils_buf[1];
            self.actuators.oxidizer_circ_vlv = coils_buf[2];
            self.actuators.fuel_circ_vlv = coils_buf[3];
            self.actuators.film_circ_vlv = coils_buf[4];
        }

        if regs_buf.len() >= 14 {
            self.pressures.pump_out_pres = decode_f32(regs_buf[0], regs_buf[1]);
            self.pressures.fuel_circ_pres = decode_f32(regs_buf[2], regs_buf[3]);
            self.pressures.oxidizer_circ_pres = decode_f32(regs_buf[4], regs_buf[5]);
            self.pressures.film_circ_pres = decode_f32(regs_buf[6], regs_buf[7]);

            self.flows.fuel_circ_mass = decode_f32(regs_buf[8], regs_buf[9]);
            self.flows.oxidizer_circ_mass = decode_f32(regs_buf[10], regs_buf[11]);
            self.flows.film_circ_mass = decode_f32(regs_buf[12], regs_buf[13]);
        }
    }
}