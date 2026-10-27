import React from 'react';
import {
  Box,
  Text,
  Paper,
  useMantineTheme,
  useMantineColorScheme,
} from "@mantine/core";
import { CONTAINER_HEIGHT, SLOT_WIDTH } from ".";

interface DigitalActuatorProps {
  onChange: (active: boolean) => void;
  enabled: boolean;
  allowed: boolean;
  defaultEnabled: boolean;
  stateText?: [string, string];
}

export function DigitalActuator({
  onChange,
  enabled,
  allowed,
  defaultEnabled,
  stateText = ["Open", "Closed"],
}: DigitalActuatorProps) {
  const { colorScheme } = useMantineColorScheme();
  const theme = useMantineTheme();

  const notNormal = defaultEnabled !== enabled;
  const buttonColorName = enabled ? "green" : "red";
  const buttonColor = theme.colors[buttonColorName][2];
  const defaultBg =
    colorScheme === "dark" ? theme.colors.dark[6] : theme.colors.gray[0];
  const backgroundColor = notNormal ? buttonColor : defaultBg;
  const borderAndTextColor = allowed
    ? theme.colors[buttonColorName][8]
    : theme.colors["dark"][2];

  return (
    <Paper
      onClick={() => {
        if (!allowed) return;
        onChange(!enabled);
      }}
      shadow="xs"
      radius="md"
      withBorder
      style={{
        position: "relative",
        backgroundColor,
        overflow: "hidden",
        cursor: allowed ? "pointer" : "not-allowed",
        height: CONTAINER_HEIGHT,
        width: `calc(1.5 * ${SLOT_WIDTH})`,
        userSelect: "none",
      }}
    >
      <Box
        style={{
          position: "absolute",
          top: 0,
          bottom: 0,
          width: `calc(${SLOT_WIDTH})`,
          zIndex: 2,
          borderRadius: theme.radius.md,
          borderWidth: 2,
          borderStyle: "solid",
          borderColor: borderAndTextColor,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          transition:
            "transform 0.4s ease-in-out, background-color 0.3s ease, color 0.3s ease, border-color 0.3s ease",
          backgroundColor: buttonColor,
          color: borderAndTextColor,
          transform: notNormal
            ? `translateX(calc(${SLOT_WIDTH} * 0.5))`
            : "translateX(0)",
        }}
      >
        <Text p="xs" size="sm" fw={600}>
          {stateText[enabled ? 0 : 1]}
        </Text>
      </Box>
    </Paper>
  );
}