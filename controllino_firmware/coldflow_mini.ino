#include <Controllino.h>
#include <ArduinoRS485.h>
#include <ArduinoModbus.h>

// --- Modbus Addresses ---
#define SLAVE_ID          1
#define COIL_BASE         0x100
#define COIL_COUNT        5      // 0x100 (pump_power) .. 0x104 (film_circ_vlv)
#define HOLDING_REG_BASE  0x300
#define HOLDING_REG_COUNT 14     // 0x300..0x307 (4x pressure), 0x308..0x30D (3x flow)

// --- Actuator Relay Pins ---
const int PIN_PUMP_PWR      = CONTROLLINO_D0; // 0x100: pump_power (Relay R0)
const int PIN_PUMP_OUT_VLV  = CONTROLLINO_D5; // 0x101: pump_out_vlv (Relay R5)
const int PIN_OX_CIRC_VLV   = CONTROLLINO_D4; // 0x102: oxidizer_circ_vlv (Relay R4)
const int PIN_FUEL_CIRC_VLV = CONTROLLINO_D3; // 0x103: fuel_circ_vlv (Relay R3)
const int PIN_FILM_CIRC_VLV = CONTROLLINO_D2; // 0x104: film_circ_vlv (Relay R2)

// --- Pressure Sensor Pins (0-25 bar, 4-20mA over 500 Ohm -> 2-10V) ---
const int PIN_PRES_PUMP_OUT = CONTROLLINO_A2; // 0x300-0x301: pump_out_pres
const int PIN_PRES_FUEL     = CONTROLLINO_A3; // 0x302-0x303: fuel_circ_pres
const int PIN_PRES_OX       = CONTROLLINO_A4; // 0x304-0x305: oxidizer_circ_pres
const int PIN_PRES_FILM     = CONTROLLINO_A5; // 0x306-0x307: film_circ_pres

// --- Flow Sensor Pins (16.0 - 90.2 Hz -> 1.0 - 30.0 L/min) ---
const int PIN_FLOW_FUEL = CONTROLLINO_A1;  // 0x308-0x309: fuel_circ_mass (PCINT on PC1)
const int PIN_FLOW_OX   = CONTROLLINO_IN1; // 0x30A-0x30B: oxidizer_circ_mass (INT1 / Pin 3)
const int PIN_FLOW_FILM = CONTROLLINO_IN0; // 0x30C-0x30D: film_circ_mass (INT0 / Pin 2)

// --- Pulse Counters ---
volatile unsigned long pulsesFuel = 0;
volatile unsigned long pulsesOx   = 0;
volatile unsigned long pulsesFilm = 0;

volatile uint8_t lastPinStateA1 = 0;
unsigned long lastFlowCalcMs = 0;

// ISR for Pin Change on Port C (tracks A1 rising edge)
ISR(PCINT1_vect) {
    uint8_t currentState = PINC & (1 << PC1);
    if (currentState && !lastPinStateA1) {
        pulsesFuel++;
    }
    lastPinStateA1 = currentState;
}

// Hardware external interrupt ISRs
void isrOxFlow() {
    pulsesOx++;
}

void isrFilmFlow() {
    pulsesFilm++;
}

// Write IEEE-754 Big-Endian Float to 2 Holding Registers
void writeFloat(int baseAddr, float value) {
    uint32_t bits;
    memcpy(&bits, &value, 4);
    ModbusRTUServer.holdingRegisterWrite(baseAddr,     (uint16_t)(bits >> 16));
    ModbusRTUServer.holdingRegisterWrite(baseAddr + 1, (uint16_t)(bits & 0xFFFF));
}

// Convert 2-10V analog reading to 0-25 bar
float readPressure(int pin) {
    float voltage = analogRead(pin) * (30.7f / 1023.0f);
    float pressure = ((voltage - 2.0f) / 8.0f) * 25.0f;
    return (pressure < 0.0f) ? 0.0f : pressure;
}

// Convert pulse count to L/min
float calcFlowLmin(unsigned long pulses, float dtSec) {
    float freqHz = pulses / dtSec;
    if (freqHz < 1.0f) return 0.0f;
    float flow = 1.0f + ((freqHz - 16.0f) / (90.2f - 16.0f)) * 29.0f;
    return (flow < 0.0f) ? 0.0f : flow;
}

void setup() {
    // 1. Actuator output configuration
    pinMode(PIN_PUMP_PWR, OUTPUT);
    digitalWrite(PIN_PUMP_PWR, LOW);
    pinMode(PIN_PUMP_OUT_VLV, OUTPUT);
    digitalWrite(PIN_PUMP_OUT_VLV, LOW);
    pinMode(PIN_OX_CIRC_VLV, OUTPUT);
    digitalWrite(PIN_OX_CIRC_VLV, LOW);
    pinMode(PIN_FUEL_CIRC_VLV, OUTPUT);
    digitalWrite(PIN_FUEL_CIRC_VLV, LOW);
    pinMode(PIN_FILM_CIRC_VLV, OUTPUT);
    digitalWrite(PIN_FILM_CIRC_VLV, LOW);

    // 2. Sensor input configuration
    pinMode(PIN_PRES_PUMP_OUT, INPUT);
    pinMode(PIN_PRES_FUEL, INPUT);
    pinMode(PIN_PRES_OX, INPUT);
    pinMode(PIN_PRES_FILM, INPUT);

    pinMode(PIN_FLOW_FUEL, INPUT);
    pinMode(PIN_FLOW_OX, INPUT);
    pinMode(PIN_FLOW_FILM, INPUT);

    // Attach PCINT on A1 (PCINT9) for fuel flow
    PCICR  |= (1 << PCIE1);
    PCMSK1 |= (1 << PCINT9);

    // Attach external interrupts for oxidizer and film flow
    attachInterrupt(digitalPinToInterrupt(PIN_FLOW_OX), isrOxFlow, RISING);
    attachInterrupt(digitalPinToInterrupt(PIN_FLOW_FILM), isrFilmFlow, RISING);

    // 3. Modbus RTU Server over USB Serial
    Serial.begin(115200);
    if (!ModbusRTUServer.begin(SLAVE_ID, 115200)) {
        while (1);
    }

    ModbusRTUServer.configureCoils(COIL_BASE, COIL_COUNT);
    ModbusRTUServer.configureHoldingRegisters(HOLDING_REG_BASE, HOLDING_REG_COUNT);
}

void loop() {
    // 1. Poll Modbus requests
    ModbusRTUServer.poll();

    // 2. Update Actuators from Coils
    digitalWrite(PIN_PUMP_PWR,      ModbusRTUServer.coilRead(0x100) ? HIGH : LOW);
    digitalWrite(PIN_PUMP_OUT_VLV,  ModbusRTUServer.coilRead(0x101) ? HIGH : LOW);
    digitalWrite(PIN_OX_CIRC_VLV,   ModbusRTUServer.coilRead(0x102) ? HIGH : LOW);
    digitalWrite(PIN_FUEL_CIRC_VLV, ModbusRTUServer.coilRead(0x103) ? HIGH : LOW);
    digitalWrite(PIN_FILM_CIRC_VLV, ModbusRTUServer.coilRead(0x104) ? HIGH : LOW);

    // 3. Read Pressure Sensors
    writeFloat(0x300, readPressure(PIN_PRES_PUMP_OUT));
    writeFloat(0x302, readPressure(PIN_PRES_FUEL));
    writeFloat(0x304, readPressure(PIN_PRES_OX));
    writeFloat(0x306, readPressure(PIN_PRES_FILM));

    // 4. Calculate Mass Flows (every 250 ms)
    unsigned long now = millis();
    if (now - lastFlowCalcMs >= 250) {
        noInterrupts();
        unsigned long pFuel = pulsesFuel; pulsesFuel = 0;
        unsigned long pOx   = pulsesOx;   pulsesOx   = 0;
        unsigned long pFilm = pulsesFilm; pulsesFilm = 0;
        interrupts();

        float dtSec = (now - lastFlowCalcMs) / 1000.0f;
        lastFlowCalcMs = now;

        writeFloat(0x308, calcFlowLmin(pFuel, dtSec));
        writeFloat(0x30A, calcFlowLmin(pOx,   dtSec));
        writeFloat(0x30C, calcFlowLmin(pFilm, dtSec));
    }
}