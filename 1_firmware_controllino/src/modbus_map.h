#pragma once
#include <stdint.h>

// ============================================================================
// COILS (Read / Write Discrete Booleans)
// ============================================================================
static const uint16_t ADDR_COIL_PUMP_POWER        = 0x0100;
static const uint16_t ADDR_COIL_PUMP_OUT_VLV      = 0x0101;
static const uint16_t ADDR_COIL_OXIDIZER_CIRC_VLV = 0x0102;
static const uint16_t ADDR_COIL_FUEL_CIRC_VLV     = 0x0103;
static const uint16_t ADDR_COIL_FILM_CIRC_VLV     = 0x0104;

static const uint16_t COIL_START_ADDR             = 0x0100;
static const uint16_t COIL_COUNT                  = 5;

// ============================================================================
// HOLDING REGISTERS (Read-Only Telemetry, 32-bit Floats = 2 x 16-bit Regs)
// ============================================================================
// Pressures [bar] (0x0300 - 0x0307)
static const uint16_t ADDR_REG_PUMP_OUT_PRES      = 0x0300; // 0x300, 0x301
static const uint16_t ADDR_REG_FUEL_CIRC_PRES     = 0x0302; // 0x302, 0x303
static const uint16_t ADDR_REG_OXIDIZER_CIRC_PRES = 0x0304; // 0x304, 0x305
static const uint16_t ADDR_REG_FILM_CIRC_PRES     = 0x0306; // 0x306, 0x307

// Mass Flow Rates [L/min] (0x0308 - 0x030D)
static const uint16_t ADDR_REG_FUEL_CIRC_MASS     = 0x0308; // 0x308, 0x309
static const uint16_t ADDR_REG_OXIDIZER_CIRC_MASS = 0x030A; // 0x30A, 0x30B
static const uint16_t ADDR_REG_FILM_CIRC_MASS     = 0x030C; // 0x30C, 0x30D

static const uint16_t HOLDING_REG_START_ADDR      = 0x0300;
static const uint16_t HOLDING_REG_COUNT           = 14; // 7 sensors * 2 registers

// Helper: Pack 32-bit float into two 16-bit Modbus registers (Big-Endian / AB CD)
inline void pack_float_to_regs(float val, uint16_t* regs, uint16_t offset) {
    union {
        float f;
        uint16_t w[2];
    } u;
    u.f = val;
    // Standard Modbus float order (High word first, Low word second)
    regs[offset]     = u.w[1];
    regs[offset + 1] = u.w[0];
}