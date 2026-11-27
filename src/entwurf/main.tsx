import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MotionRoot } from "../ui";
import { Entwurf } from "./Entwurf";
import "../index.css";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <MotionRoot>
        <Entwurf />
      </MotionRoot>
    </StrictMode>,
  );
}
