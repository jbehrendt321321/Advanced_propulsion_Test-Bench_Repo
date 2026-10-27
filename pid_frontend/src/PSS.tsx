import React, { CSSProperties, useEffect, useRef, useState } from "react";
import { ActionIcon, Affix, Badge, Box, Button, Flex, Text, Tooltip } from "@mantine/core";
import { useToggle, useViewportSize } from "@mantine/hooks";
import * as d3 from "d3";
import { IconEye, IconEyeOff, IconZoomReset } from "@tabler/icons-react";
// @ts-ignore
import classes from "./PSS.module.css";
import { AnalogSensor } from "./datapoint/AnalogSensor";
import { DigitalActuator } from "./datapoint/DigitalActuator";

const CELL_ID_ATTR = "data-cell-id";
const API_URL = "http://localhost:3001/api";

type Transform = { k: number; x: number; y: number };

interface DatapointMeta {
  cell: string;
  kind: "sensor" | "actuator";
  label: string;
  unit?: string;
  precision?: number;
  align: "left" | "right" | "hidden";
  stateText?: [string, string];
}

// All 12 components mapped directly to the SVG's `data-cell-id` abbreviations
const DATAPOINTS: Record<string, DatapointMeta> = {
  // Actuators
  pump_power:         { cell: "pump_power",         kind: "actuator", label: "PUMP",  stateText: ["On", "Off"],     align: "left" },
  pump_out_vlv:       { cell: "pump_out_vlv",       kind: "actuator", label: "SV-1",  stateText: ["Open", "Closed"], align: "left" },
  oxidizer_circ_vlv:  { cell: "oxidizer_circ_vlv",  kind: "actuator", label: "SV-OX", stateText: ["Open", "Closed"], align: "left" },
  fuel_circ_vlv:      { cell: "fuel_circ_vlv",      kind: "actuator", label: "SV-FL", stateText: ["Open", "Closed"], align: "left" },
  film_circ_vlv:      { cell: "film_circ_vlv",      kind: "actuator", label: "SV-FC", stateText: ["Open", "Closed"], align: "left" },

  // Pressure Sensors (0 - 25 bar)
  pump_out_pres:      { cell: "pump_out_pres",      kind: "sensor",   label: "PT-1",  unit: "bar",   precision: 2,   align: "left" },
  fuel_circ_pres:     { cell: "fuel_circ_pres",     kind: "sensor",   label: "PT-2",  unit: "bar",   precision: 2,   align: "left" },
  oxidizer_circ_pres: { cell: "oxidizer_circ_pres", kind: "sensor",   label: "PT-3",  unit: "bar",   precision: 2,   align: "left" },
  film_circ_pres:     { cell: "film_circ_pres",     kind: "sensor",   label: "PT-4",  unit: "bar",   precision: 2,   align: "left" },

  // Mass Flow Sensors (1 - 30 L/min)
  fuel_circ_mass:     { cell: "fuel_circ_mass",     kind: "sensor",   label: "FM-1",  unit: "L/min", precision: 1,   align: "left" },
  oxidizer_circ_mass: { cell: "oxidizer_circ_mass", kind: "sensor",   label: "FM-2",  unit: "L/min", precision: 1,   align: "left" },
  film_circ_mass:     { cell: "film_circ_mass",     kind: "sensor",   label: "FM-3",  unit: "L/min", precision: 1,   align: "left" },
};

interface BenchState {
  online: boolean;
  pump_power: boolean;
  pump_out_vlv: boolean;
  oxidizer_circ_vlv: boolean;
  fuel_circ_vlv: boolean;
  film_circ_vlv: boolean;
  pump_out_pres: number;
  fuel_circ_pres: number;
  oxidizer_circ_pres: number;
  film_circ_pres: number;
  fuel_circ_mass: number;
  oxidizer_circ_mass: number;
  film_circ_mass: number;
}

const DEFAULT_STATE: BenchState = {
  online: false,
  pump_power: false,
  pump_out_vlv: false,
  oxidizer_circ_vlv: false,
  fuel_circ_vlv: false,
  film_circ_vlv: false,
  pump_out_pres: 0,
  fuel_circ_pres: 0,
  oxidizer_circ_pres: 0,
  film_circ_pres: 0,
  fuel_circ_mass: 0,
  oxidizer_circ_mass: 0,
  film_circ_mass: 0,
};

export default function PSS() {
  const { width, height } = useViewportSize();
  const [container, setContainer] = useState<HTMLElement | null>(null);
  const [svgReady, setSvgReady] = useState(false);
  const [zoomTransform, setZoomTransform] = useState<Transform>({ k: 1, x: 0, y: 0 });
  const [datapointVisibility, toggleDatapointVisibility] = useToggle(["hidden", "inherit"]);

  const [fakeHw, setFakeHw] = useState(false);
  const fakeHwRef = useRef(false);

  const toggleFakeHw = () => {
    fakeHwRef.current = !fakeHwRef.current;
    setFakeHw(fakeHwRef.current);
  };

  const [bench, setBench] = useState<BenchState>(DEFAULT_STATE);

  // 1. Poll Rust Backend at 20 Hz
  useEffect(() => {
    const timer = setInterval(async () => {
      try {
        const res = await fetch(`${API_URL}/state`);
        if (res.ok) {
          const data: BenchState = await res.json();
          setBench((prev) => ({
            ...data,
            online: fakeHwRef.current ? true : data.online,
            pump_power: fakeHwRef.current ? prev.pump_power : data.pump_power,
            pump_out_vlv: fakeHwRef.current ? prev.pump_out_vlv : data.pump_out_vlv,
            oxidizer_circ_vlv: fakeHwRef.current ? prev.oxidizer_circ_vlv : data.oxidizer_circ_vlv,
            fuel_circ_vlv: fakeHwRef.current ? prev.fuel_circ_vlv : data.fuel_circ_vlv,
            film_circ_vlv: fakeHwRef.current ? prev.film_circ_vlv : data.film_circ_vlv,
          }));
        }
      } catch {
        setBench((prev) => ({
          ...prev,
          online: fakeHwRef.current,
        }));
      }
    }, 50);
    return () => clearInterval(timer);
  }, []);

  // 2. Load SVG and attach D3 Zoom/Pan
  useEffect(() => {
    if (!container) return;

    fetch("/Test-Bench_PID_V1.svg")
      .then((res) => res.text())
      .then((svgText) => {
        container.innerHTML = svgText;
        const svg = d3.select(container).select<SVGSVGElement>("svg");
        svg.attr("width", "100%").attr("height", "100%");

        const svgNode = svg.node() as SVGSVGElement | null;
        const children = Array.from(svgNode?.childNodes || []);
        const g = svg.append("g").attr("class", "zoom-layer");
        const gNode = g.node() as SVGGElement | null;
        if (gNode) {
          children.forEach((child) => gNode.appendChild(child as Node));
        }

        const zoom = d3.zoom<SVGSVGElement, unknown>().on("zoom", (event) => {
          const t: Transform = event.transform;
          g.attr("transform", event.transform.toString());
          setZoomTransform(t);

          Object.entries(DATAPOINTS).forEach(([name, info]) => {
            const styles = computeDatapointStyle(t, info);
            if (!styles) return;
            const el = document.querySelector(
              `[data-datapoint-container-for="${name}"]`
            ) as HTMLDivElement | null;
            if (!el) return;
            el.style.setProperty("top", styles[0].top as string);
            el.style.setProperty("left", styles[0].left as string);
            const inner = el.firstElementChild as HTMLDivElement | null;
            if (inner) inner.style.setProperty("transform", styles[1].transform as string);
          });
        });

        svg.call(zoom as any);
        (container as any).__resetZoom = () =>
          svg.transition().duration(250).call(zoom.transform as any, d3.zoomIdentity);

        setSvgReady(true);
      });
  }, [container]);

  // 3. Toggle underlying SVG anchor boxes
  useEffect(() => {
    if (!svgReady) return;
    Object.values(DATAPOINTS).forEach(({ cell }) => {
      d3.select(`[${CELL_ID_ATTR}="${cell}"]`).style("visibility", datapointVisibility);
    });
  }, [datapointVisibility, svgReady]);

  async function triggerActuator(name: string, value: boolean) {
    setBench((prev) => ({ ...prev, [name]: value }));

    try {
      await fetch(`${API_URL}/actuator`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name, value }),
      });
    } catch (err) {
      // Mock mode fallback
    }
  }

  return (
    <>
      {/* Overlaid Interactive Datapoints */}
      {svgReady &&
        Object.entries(DATAPOINTS).map(([name, info]) => {
          const styles = computeDatapointStyle(zoomTransform, info);
          if (!styles) return <React.Fragment key={name} />;

          const isActuator = info.kind === "actuator";
          const rawValue = bench[name as keyof BenchState];

          return (
            <div data-datapoint-container-for={name} key={name} style={styles[0]}>
              <div style={styles[1]}>
                {isActuator ? (
                  <DigitalActuator
                    enabled={Boolean(rawValue)}
                    allowed={bench.online}
                    defaultEnabled={false}
                    stateText={info.stateText ?? ["Open", "Closed"]}
                    onChange={(newVal) => triggerActuator(name, newVal)}
                  />
                ) : (
                  <AnalogSensor
                    value={typeof rawValue === "number" ? rawValue : 0}
                    precision={info.precision ?? 2}
                    unit={info.unit ?? ""}
                  />
                )}
                {info.align !== "hidden" && (
                  <Text size="xs" fw={700} c="dimmed" ff="monospace">
                    {info.label}
                  </Text>
                )}
              </div>
            </div>
          );
        })}

      {/* Top Status & Simulation Toggle */}
      <Affix position={{ top: "1rem", left: "1rem" }}>
        <Flex gap="xs" align="center">
          <Badge size="lg" color={bench.online ? "green" : "red"}>
            {bench.online ? "CONTROLLINO ONLINE" : "CONTROLLINO OFFLINE"}
          </Badge>
          <Button
            size="xs"
            variant={fakeHw ? "filled" : "default"}
            color={fakeHw ? "teal" : "gray"}
            onClick={toggleFakeHw}
          >
            {fakeHw ? "Fake HW: ON" : "Fake HW: OFF"}
          </Button>
        </Flex>
      </Affix>

      {/* Top Right Controls */}
      <Affix position={{ top: "1rem", right: "1rem" }}>
        <Flex gap="xs">
          <Tooltip label="Toggle SVG anchor visibility">
            <ActionIcon variant="filled" onClick={() => toggleDatapointVisibility()}>
              {datapointVisibility === "inherit" ? <IconEyeOff size={18} /> : <IconEye size={18} />}
            </ActionIcon>
          </Tooltip>
          <Tooltip label="Reset zoom and pan">
            <ActionIcon variant="filled" onClick={() => (container as any)?.__resetZoom?.()}>
              <IconZoomReset size={18} />
            </ActionIcon>
          </Tooltip>
        </Flex>
      </Affix>

      {/* Main SVG Container */}
      <Box w={width} h={height}>
        <section className={classes.container} ref={setContainer} />
      </Box>
    </>
  );
}

function computeDatapointStyle(
  transform: Transform,
  info: DatapointMeta
): [CSSProperties, CSSProperties] | undefined {
  const node = d3.select(`[${CELL_ID_ATTR}="${info.cell}"]`).node() as SVGElement | null;
  if (!node) return undefined;

  const bounds = node.getBoundingClientRect();
  const containerStyle: CSSProperties = {
    position: "absolute",
    zIndex: 10,
    top: `${window.scrollY + bounds.top + bounds.height / 2}px`,
    left: `${window.scrollX + bounds.left + bounds.width / 2}px`,
    transform:
      info.align === "right" ? "translateY(-50%) translateX(-100%)" : "translateY(-50%)",
  };

  const innerStyle: CSSProperties = {
    transform: `scale(${transform.k})`,
    transformOrigin: `center ${info.align === "right" ? "right" : "left"}`,
    display: "flex",
    whiteSpace: "nowrap",
    gap: "0.4rem",
    alignItems: "center",
    flexDirection: info.align === "right" ? "row-reverse" : "row",
  };

  return [containerStyle, innerStyle];
}