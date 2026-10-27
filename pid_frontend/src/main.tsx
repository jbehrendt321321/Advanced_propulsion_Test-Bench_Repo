import React, { StrictMode } from "react";
import { createRoot } from "react-dom/client";
// @ts-ignore
import "@mantine/core/styles.css";
import { createTheme, MantineProvider, Tooltip } from "@mantine/core";
import PSS from "./PSS";

const theme = createTheme({
  components: {
    Tooltip: Tooltip.extend({
      defaultProps: { zIndex: 1500 },
    }),
  },
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <MantineProvider defaultColorScheme="light" theme={theme}>
      <PSS />
    </MantineProvider>
  </StrictMode>
);