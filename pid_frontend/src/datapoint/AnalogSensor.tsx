import React from 'react';
import {
  Text,
  Paper,
  useMantineTheme,
  useMantineColorScheme,
} from "@mantine/core";
import { CONTAINER_HEIGHT } from ".";
import { useEffect, useRef, useState } from "react";

const TENDENCY_THRESHOLD: number = 0.2;
type AnalogSensorTendency = "equal" | "grow" | "shrink";

interface AnalogSensorProps {
  value: number;
  precision: number;
  unit: string;
}

function fmt(value: number | undefined, decimals = 3): string {
  if (value == undefined || value == null || Number.isNaN(value)) return "--";
  return value.toFixed(decimals);
}

export function AnalogSensor({ value, unit, precision }: AnalogSensorProps) {
  const { colorScheme } = useMantineColorScheme();
  const theme = useMantineTheme();
  const previousValue = useRef<number | null>(null);
  const [tendency, setTendency] = useState<AnalogSensorTendency>("equal");

  useEffect(() => {
    if (previousValue.current === null) {
      previousValue.current = value;
    }
    const difference = previousValue.current - value;
    if (difference > TENDENCY_THRESHOLD) setTendency("shrink");
    else if (difference < -TENDENCY_THRESHOLD) setTendency("grow");
    else setTendency("equal");
    previousValue.current = value;
  }, [value]);

  return (
    <Paper
      shadow="xs"
      radius="md"
      withBorder
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        position: "relative",
        backgroundColor:
          colorScheme === "dark" ? theme.colors.dark[6] : theme.colors.gray[0],
        overflow: "hidden",
        height: CONTAINER_HEIGHT,
        width: "8.5rem",
      }}
    >
      <Text p="6pt" size="sm" fw={600} ff={"monospace"}>
        {fmt(value, precision)}
      </Text>
      <Tendency tendency={tendency} />
      <Text p="6pt" size="sm" fw={500}>
        {unit}
      </Text>
    </Paper>
  );
}

function Tendency({ tendency }: { tendency: AnalogSensorTendency }) {
  const theme = useMantineTheme();
  const centerY = tendency == "equal" ? 50 : tendency == "shrink" ? 80 : 20;
  const color =
    tendency == "equal" ? "blue" : tendency == "shrink" ? "orange" : "green";
  const translate =
    tendency == "equal" ? "0" : tendency == "shrink" ? "-10%" : "10%";

  return (
    <svg
      height={"1em"}
      viewBox="0 0 80 100"
      preserveAspectRatio="xMidYMid meet"
      style={{ verticalAlign: "middle", transform: `translateY(${translate})` }}
    >
      <polyline
        fill={"none"}
        stroke={theme.colors[color][5]}
        strokeWidth={15}
        strokeLinecap="round"
        points={`10,50 40,${centerY} 70,50`}
      />
    </svg>
  );
}