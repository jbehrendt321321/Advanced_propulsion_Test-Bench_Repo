use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Actuators {
    pub pump_power: bool,
    pub pump_out_vlv: bool,
    pub oxidizer_circ_vlv: bool,
    pub fuel_circ_vlv: bool,
    pub film_circ_vlv: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pressures {
    pub pump_out_pres: f32,      // [bar]
    pub fuel_circ_pres: f32,     // [bar]
    pub oxidizer_circ_pres: f32, // [bar]
    pub film_circ_pres: f32,     // [bar]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowRates {
    pub fuel_circ_mass: f32,     // [L/min]
    pub oxidizer_circ_mass: f32, // [L/min]
    pub film_circ_mass: f32,     // [L/min]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchTelemetry {
    pub timestamp_ms: u64,
    pub actuators: Actuators,
    pub pressures: Pressures,
    pub flows: FlowRates,
}

impl BenchTelemetry {
    /// Deserializes a raw 14-register holding buffer and 5-bit coil slice into typed telemetry
    pub fn from_modbus_buffers(
        coils: &[bool],
        regs: &[u16],
        timestamp_ms: u64,
    ) -> Result<Self, &'static str> {
        if coils.len() < 5 || regs.len() < 14 {
            return Err("Modbus read buffers too short for bench layout");
        }

        use crate::modbus_map::unpack_f32;

        Ok(Self {
            timestamp_ms,
            actuators: Actuators {
                pump_power: coils[0],
                pump_out_vlv: coils[1],
                oxidizer_circ_vlv: coils[2],
                fuel_circ_vlv: coils[3],
                film_circ_vlv: coils[4],
            },
            pressures: Pressures {
                pump_out_pres: unpack_f32(regs[0], regs[1]),
                fuel_circ_pres: unpack_f32(regs[2], regs[3]),
                oxidizer_circ_pres: unpack_f32(regs[4], regs[5]),
                film_circ_pres: unpack_f32(regs[6], regs[7]),
            },
            flows: FlowRates {
                fuel_circ_mass: unpack_f32(regs[8], regs[9]),
                oxidizer_circ_mass: unpack_f32(regs[10], regs[11]),
                film_circ_mass: unpack_f32(regs[12], regs[13]),
            },
        })
    }
}