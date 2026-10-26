pub mod coils {
    pub const PUMP_POWER: u16        = 0x0100;
    pub const PUMP_OUT_VLV: u16      = 0x0101;
    pub const OXIDIZER_CIRC_VLV: u16 = 0x0102;
    pub const FUEL_CIRC_VLV: u16     = 0x0103;
    pub const FILM_CIRC_VLV: u16     = 0x0104;

    pub const START_ADDR: u16 = 0x0100;
    pub const COUNT: u16 = 5;
}

pub mod holding_registers {
    pub const PUMP_OUT_PRES: u16      = 0x0300;
    pub const FUEL_CIRC_PRES: u16     = 0x0302;
    pub const OXIDIZER_CIRC_PRES: u16 = 0x0304;
    pub const FILM_CIRC_PRES: u16     = 0x0306;

    pub const FUEL_CIRC_MASS: u16     = 0x0308;
    pub const OXIDIZER_CIRC_MASS: u16 = 0x030A;
    pub const FILM_CIRC_MASS: u16     = 0x030C;

    pub const START_ADDR: u16 = 0x0300;
    pub const COUNT: u16 = 14;
}

/// Unpacks two consecutive 16-bit Modbus registers into an IEEE-754 32-bit float (AB CD word order)
pub fn unpack_f32(high_word: u16, low_word: u16) -> f32 {
    let raw = ((high_word as u32) << 16) | (low_word as u32);
    f32::from_bits(raw)
}