import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { applyTheme, storedTheme } from "./settings/theme";
import { App } from "./App";
import { MotionRoot } from "./ui";
import "./index.css";

// Light, dark or as the system, chosen in Einstellungen › Darstellung, before the first paint (#49).
applyTheme(storedTheme());

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <MotionRoot>
        <App />
      </MotionRoot>
    </StrictMode>,
  );
}
