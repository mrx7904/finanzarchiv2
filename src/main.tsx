import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { applyTheme, storedTheme } from "./settings/theme";
import { App } from "./App";
import { MotionRoot, applySize, readSize } from "./ui";
import "./index.css";

// Light, dark or as the system, chosen in Einstellungen › Darstellung, before the first paint (#49).
applyTheme(storedTheme());

// The stand-in for the core, in the dev server only (src/dev/mock.ts).
if (import.meta.env.DEV && new URLSearchParams(window.location.search).has("mock")) await import("./dev/mock");

// The size of the view, Klein, Mittel or Groß, chosen in Einstellungen › Darstellung or by Strg with +, − and 0 (#53).
void applySize(readSize());

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
