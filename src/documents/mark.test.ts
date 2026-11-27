// #57: the mark lies on the image of the page, not on its box, at "Ganze Seite" as at every zoom.
import { describe, expect, it } from "vitest";
import { containedRect, markRect } from "./pages";

const inside = (outer: { left: number; top: number; width: number; height: number }, inner: typeof outer) =>
  inner.left >= outer.left - 0.01 && inner.top >= outer.top - 0.01 && inner.left + inner.width <= outer.left + outer.width + 0.01 && inner.top + inner.height <= outer.top + outer.height + 0.01;

describe("the mark of a reading (#57)", () => {
  it("stands on a portrait page fitted into a wide box", () => {
    const box = { width: 900, height: 600 };
    const image = containedRect(box, { width: 1280, height: 1810 });
    expect(image.height).toBeCloseTo(600);
    expect(image.width).toBeCloseTo(424.3, 1);
    expect(image.left).toBeCloseTo((900 - 424.3) / 2, 0);
    const mark = markRect(image, [0.08, 0.46, 0.5, 0.03]);
    expect(mark.left).toBeCloseTo(image.left + 0.08 * image.width);
    expect(mark.top).toBeCloseTo(0.46 * 600);
    expect(mark.width).toBeCloseTo(0.5 * image.width);
    expect(inside(image, mark)).toBe(true);
    // Placed by the box as in 0.7.0, the mark would begin 72 px left of the page.
    expect(0.08 * box.width).toBeLessThan(image.left);
  });

  it("stands on a landscape page fitted into a tall box", () => {
    const image = containedRect({ width: 500, height: 900 }, { width: 1280, height: 905 });
    expect(image.width).toBeCloseTo(500);
    expect(image.top).toBeCloseTo((900 - (905 * 500) / 1280) / 2);
    const mark = markRect(image, [0.9, 0.95, 0.1, 0.05]);
    expect(inside(image, mark)).toBe(true);
    expect(mark.top + mark.height).toBeCloseTo(image.top + image.height);
  });

  it("does not enlarge an image smaller than its box, as max-width and max-height do not", () => {
    expect(containedRect({ width: 1000, height: 1000 }, { width: 640, height: 905 })).toEqual({ left: 180, top: 47.5, width: 640, height: 905 });
    expect(containedRect({ width: 0, height: 100 }, { width: 640, height: 905 }).width).toBe(0);
  });
});
