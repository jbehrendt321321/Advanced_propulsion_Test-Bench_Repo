#pragma once
#include <Controllino.h>

// --- Actuator Outputs ---
#define PIN_PUMP_POWER        CONTROLLINO_R0 // 230V Relay 0: Comet MC 20/20 pump power
#define PIN_PUMP_OUT_VLV      CONTROLLINO_D0 // 24V Out 0: Main outlet solenoid
#define PIN_OXIDIZER_CIRC_VLV CONTROLLINO_D1 // 24V Out 1: Oxidizer circuit valve
#define PIN_FUEL_CIRC_VLV     CONTROLLINO_D2 // 24V Out 2: Fuel circuit valve
#define PIN_FILM_CIRC_VLV     CONTROLLINO_D3 // 24V Out 3: Film cooling valve

// --- Analog Sensor Inputs (0 - 10V) ---
#define PIN_PUMP_OUT_PRES     CONTROLLINO_A0 // Pressure: Pump supply
#define PIN_FUEL_CIRC_PRES    CONTROLLINO_A1 // Pressure: Fuel line
#define PIN_OXIDIZER_CIRC_PRES CONTROLLINO_A2 // Pressure: Oxidizer line
#define PIN_FILM_CIRC_PRES    CONTROLLINO_A3 // Pressure: Film cooling line

#define PIN_FUEL_CIRC_MASS    CONTROLLINO_A4 // Flowmeter: Fuel line
#define PIN_OXIDIZER_CIRC_MASS CONTROLLINO_A5 // Flowmeter: Oxidizer line
#define PIN_FILM_CIRC_MASS    CONTROLLINO_A6 // Flowmeter: Film cooling line