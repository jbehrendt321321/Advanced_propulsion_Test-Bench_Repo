export type ActuatorKey =
  | "pump_power"
  | "pump_out_vlv"
  | "oxidizer_circ_vlv"
  | "fuel_circ_vlv"
  | "film_circ_vlv";

export type SensorKey =
  | "pump_out_pres"
  | "fuel_circ_pres"
  | "oxidizer_circ_pres"
  | "film_circ_pres"
  | "fuel_circ_mass"
  | "oxidizer_circ_mass"
  | "film_circ_mass";

export type DatapointKey = ActuatorKey | SensorKey;

export interface DatapointConfig {
  cell: string;
  kind: "actuator" | "sensor";
  label?: string;
  unit?: string;
  precision?: number;
  align: "left" | "right";
}

export const PID_DATAPOINTS: Record<DatapointKey, DatapointConfig> = {
  // Actuators (Coils 0x100 - 0x104)
  pump_power:        { cell: "pump_power",        kind: "actuator", label: "Pump",      align: "right" },
  pump_out_vlv:      { cell: "pump_out_vlv",      kind: "actuator", label: "Main Vlv",  align: "left"  },
  oxidizer_circ_vlv: { cell: "oxidizer_circ_vlv", kind: "actuator", label: "OX Vlv",    align: "left"  },
  fuel_circ_vlv:     { cell: "fuel_circ_vlv",     kind: "actuator", label: "Fuel Vlv",  align: "left"  },
  film_circ_vlv:     { cell: "film_circ_vlv",     kind: "actuator", label: "Film Vlv",  align: "left"  },

  // Pressure Sensors (Holding Regs 0x300 - 0x307)
  pump_out_pres:      { cell: "pump_out_pres",      kind: "sensor", unit: "bar",   precision: 2, align: "right" },
  fuel_circ_pres:     { cell: "fuel_circ_pres",     kind: "sensor", unit: "bar",   precision: 2, align: "right" },
  oxidizer_circ_pres: { cell: "oxidizer_circ_pres", kind: "sensor", unit: "bar",   precision: 2, align: "right" },
  film_circ_pres:     { cell: "film_circ_pres",     kind: "sensor", unit: "bar",   precision: 2, align: "right" },

  // Mass Flow Sensors (Holding Regs 0x308 - 0x30D)
  fuel_circ_mass:     { cell: "fuel_circ_mass",     kind: "sensor", unit: "L/min", precision: 1, align: "left"  },
  oxidizer_circ_mass: { cell: "oxidizer_circ_mass", kind: "sensor", unit: "L/min", precision: 1, align: "left"  },
  film_circ_mass:     { cell: "film_circ_mass",     kind: "sensor", unit: "L/min", precision: 1, align: "left"  },
};