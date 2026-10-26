pub mod coils {
    pub const BASE: u16 = 0x0100;
    pub const COUNT: u16 = 5;

    pub const PUMP_POWER: u16        = 0x0100;
    pub const PUMP_OUT_VLV: u16      = 0x0101;
    pub const OXIDIZER_CIRC_VLV: u16 = 0x0102;
    pub const FUEL_CIRC_VLV: u16     = 0x0103;
    pub const FILM_CIRC_VLV: u16     = 0x0104;
}

pub mod holding_registers {
    pub const BASE: u16 = 0x0300;
    pub const COUNT: u16 = 14; // 7 sensors * 2 registers each

    // Pressures [bar]
    pub const PUMP_OUT_PRES: u16      = 0x0300;
    pub const FUEL_CIRC_PRES: u16     = 0x0302;
    pub const OXIDIZER_CIRC_PRES: u16 = 0x0304;
    pub const FILM_CIRC_PRES: u16     = 0x0306;

    // Mass Flow Rates [L/min]
    pub const FUEL_CIRC_MASS: u16     = 0x0308;
    pub const OXIDIZER_CIRC_MASS: u16 = 0x030A;
    pub const FILM_CIRC_MASS: u16     = 0x030C;
}

/// Decodes two 16-bit big-endian Modbus registers into an IEEE-754 32-bit float.
/// Word order matches Asteria's writeFloat(): high word first, low word second.
pub fn decode_f32(high: u16, low: u16) -> f32 {
    let bits = ((high as u32) << 16) | (low as u32);
    f32::from_bits(bits)
}

/// Encodes a 32-bit float into two 16-bit registers (for mock/simulation).
pub fn encode_f32(val: f32) -> (u16, u16) {
    let bits = val.to_bits();
    ((bits >> 16) as u16, (bits & 0xFFFF) as u16)
}