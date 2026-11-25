import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { MotionRoot } from "./ui";
import "./index.css";

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
