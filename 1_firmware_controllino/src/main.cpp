#include <Arduino.h>
#include <Controllino.h>
#include <ArduinoRS485.h>
#include <ArduinoModbus.h>

#include "hardware_pins.h"
#include "modbus_map.h"

// Telemetry buffer
struct BenchSensors {
    float pump_out_pres;
    float fuel_circ_pres;
    float oxidizer_circ_pres;
    float film_circ_pres;
    float fuel_circ_mass;
    float oxidizer_circ_mass;
    float film_circ_mass;
} live_sensors;

// Helper: Writes IEEE-754 32-bit float across two 16-bit Modbus registers
// Big-endian AB CD word order matches Rust's decode_f32()
void writeFloatRegister(int baseAddr, float value) {
    uint32_t bits;
    memcpy(&bits, &value, 4);
    ModbusRTUServer.holdingRegisterWrite(baseAddr,     (uint16_t)(bits >> 16));
    ModbusRTUServer.holdingRegisterWrite(baseAddr + 1, (uint16_t)(bits & 0xFFFF));
}

// Converts Controllino analog input (0-10V / ADC 0-1023) to scaled physical value
float readScaledInput(int pin, float max_value) {
    int raw = analogRead(pin);
    float scaled = (raw / SENSOR_ADC_MAX) * max_value;
    return scaled < 0.0f ? 0.0f : scaled;
}

// Reads all 7 bench sensors and mirrors them into Modbus holding registers
void updateSensors() {
    live_sensors.pump_out_pres      = readScaledInput(PIN_PUMP_OUT_PRES, PRESSURE_MAX_BAR);
    live_sensors.fuel_circ_pres     = readScaledInput(PIN_FUEL_CIRC_PRES, PRESSURE_MAX_BAR);
    live_sensors.oxidizer_circ_pres = readScaledInput(PIN_OXIDIZER_CIRC_PRES, PRESSURE_MAX_BAR);
    live_sensors.film_circ_pres     = readScaledInput(PIN_FILM_CIRC_PRES, PRESSURE_MAX_BAR);

    live_sensors.fuel_circ_mass     = readScaledInput(PIN_FUEL_CIRC_MASS, FLOW_MAX_LPM);
    live_sensors.oxidizer_circ_mass = readScaledInput(PIN_OXIDIZER_CIRC_MASS, FLOW_MAX_LPM);
    live_sensors.film_circ_mass     = readScaledInput(PIN_FILM_CIRC_MASS, FLOW_MAX_LPM);

    // Holding registers 0x0300 - 0x030D (14 registers)
    writeFloatRegister(REG_PUMP_OUT_PRES,      live_sensors.pump_out_pres);
    writeFloatRegister(REG_FUEL_CIRC_PRES,     live_sensors.fuel_circ_pres);
    writeFloatRegister(REG_OXIDIZER_CIRC_PRES, live_sensors.oxidizer_circ_pres);
    writeFloatRegister(REG_FILM_CIRC_PRES,     live_sensors.film_circ_pres);

    writeFloatRegister(REG_FUEL_CIRC_MASS,     live_sensors.fuel_circ_mass);
    writeFloatRegister(REG_OXIDIZER_CIRC_MASS, live_sensors.oxidizer_circ_mass);
    writeFloatRegister(REG_FILM_CIRC_MASS,     live_sensors.film_circ_mass);
}

// Direct 1:1 hardware driving from Modbus coils (no software override)
void applyActuators() {
    digitalWrite(PIN_PUMP_POWER,          ModbusRTUServer.coilRead(COIL_PUMP_POWER) ? HIGH : LOW);
    digitalWrite(PIN_PUMP_OUT_VLV,        ModbusRTUServer.coilRead(COIL_PUMP_OUT_VLV) ? HIGH : LOW);
    digitalWrite(PIN_OXIDIZER_CIRC_VLV,   ModbusRTUServer.coilRead(COIL_OXIDIZER_CIRC_VLV) ? HIGH : LOW);
    digitalWrite(PIN_FUEL_CIRC_VLV,       ModbusRTUServer.coilRead(COIL_FUEL_CIRC_VLV) ? HIGH : LOW);
    digitalWrite(PIN_FILM_CIRC_VLV,       ModbusRTUServer.coilRead(COIL_FILM_CIRC_VLV) ? HIGH : LOW);
}

void setup() {
    // 1. Digital outputs boot LOW by default
    pinMode(PIN_PUMP_POWER, OUTPUT);
    pinMode(PIN_PUMP_OUT_VLV, OUTPUT);
    pinMode(PIN_OXIDIZER_CIRC_VLV, OUTPUT);
    pinMode(PIN_FUEL_CIRC_VLV, OUTPUT);
    pinMode(PIN_FILM_CIRC_VLV, OUTPUT);

    digitalWrite(PIN_PUMP_POWER, LOW);
    digitalWrite(PIN_PUMP_OUT_VLV, LOW);
    digitalWrite(PIN_OXIDIZER_CIRC_VLV, LOW);
    digitalWrite(PIN_FUEL_CIRC_VLV, LOW);
    digitalWrite(PIN_FILM_CIRC_VLV, LOW);

    // 2. Analog inputs
    pinMode(PIN_PUMP_OUT_PRES, INPUT);
    pinMode(PIN_FUEL_CIRC_PRES, INPUT);
    pinMode(PIN_OXIDIZER_CIRC_PRES, INPUT);
    pinMode(PIN_FILM_CIRC_PRES, INPUT);
    pinMode(PIN_FUEL_CIRC_MASS, INPUT);
    pinMode(PIN_OXIDIZER_CIRC_MASS, INPUT);
    pinMode(PIN_FILM_CIRC_MASS, INPUT);

    // 3. Configure RS485 transceiver for USB serial pass-through
    RS485.setPins(-1, -1, -1);

    // 4. Modbus RTU Server (Slave ID: 1, 115200 baud)
    if (!ModbusRTUServer.begin(1, 115200)) {
        while (1) {
            digitalWrite(PIN_PUMP_OUT_VLV, HIGH);
            delay(100);
            digitalWrite(PIN_PUMP_OUT_VLV, LOW);
            delay(100);
        }
    }

    // Allocate coil (0x100 - 0x104) and holding register (0x300 - 0x30D) tables
    ModbusRTUServer.configureCoils(COIL_BASE, COIL_COUNT);
    ModbusRTUServer.configureHoldingRegisters(HOLDING_REG_BASE, HOLDING_REG_COUNT);
}

void loop() {
    // 1. Poll incoming Modbus RTU packets over USB
    ModbusRTUServer.poll();

    // 2. Read analog sensors and push to holding registers
    updateSensors();

    // 3. Drive physical pins directly to match coil requests
    applyActuators();
}