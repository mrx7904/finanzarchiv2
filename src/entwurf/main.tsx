import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MotionRoot } from "../ui";
import { Seite } from "./Seite";
import "../index.css";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <MotionRoot>
        <Seite />
      </MotionRoot>
    </StrictMode>,
  );
}
