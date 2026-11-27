import { describe, expect, it } from "vitest";
import { pageStep, requestWidth, zoomLabel, zoomStep } from "./pages";

describe("turning pages and zooming (#54)", () => {
  it("stops at the first and the last page", () => {
    expect(pageStep(1, 3, -1)).toBe(1);
    expect(pageStep(2, 3, 1)).toBe(3);
    expect(pageStep(3, 3, 1)).toBe(3);
    expect(pageStep(1, 0, 1)).toBe(1);
  });

  it("zooms from the whole page in steps and stops at the ends", () => {
    expect(zoomStep("fit", -1)).toBe("fit");
    expect(zoomStep("fit", 1)).toBe(50);
    expect(zoomStep(200, 1)).toBe(300);
    expect(zoomStep(300, 1)).toBe(300);
    expect(zoomLabel("fit")).toBe("Ganze Seite");
    expect(zoomLabel(125)).toBe("125 %");
  });

  it("asks the core for the width the page is shown at", () => {
    expect(requestWidth(700, "fit", 1.5)).toBe(1050);
    expect(requestWidth(700, 200, 1.5)).toBe(2100);
    expect(requestWidth(40, "fit", 1)).toBe(160);
  });
});
