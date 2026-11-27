import { describe, expect, it } from "vitest";
import { clampWidth, readFlag, readNumber, readSize, sizeStep, write } from "./layout";

function memory() {
  const map = new Map<string, string>();
  return { getItem: (key: string) => map.get(key) ?? null, setItem: (key: string, value: string) => void map.set(key, value), removeItem: (key: string) => void map.delete(key) };
}

describe("what the owner fits in a view (#53)", () => {
  it("holds a column between its least and greatest width", () => {
    expect(clampWidth(100, 280, 520)).toBe(280);
    expect(clampWidth(900, 280, 520)).toBe(520);
    expect(clampWidth(400.4, 280, 520)).toBe(400);
    // A window too narrow for the greatest width still keeps the least.
    expect(clampWidth(400, 280, 200)).toBe(280);
  });

  it("keeps widths, flags and the size over a restart, with defaults for nothing or something broken", () => {
    const store = memory();
    expect(readNumber("documents.list", 360, store)).toBe(360);
    write("documents.list", 420, store);
    expect(readNumber("documents.list", 360, store)).toBe(420);
    store.setItem("finanzarchiv.layout.broken", "abc");
    expect(readNumber("broken", 7, store)).toBe(7);
    write("rail", true, store);
    expect(readFlag("rail", false, store)).toBe(true);
    write("rail", null, store);
    expect(readFlag("rail", false, store)).toBe(false);
    expect(readSize(store)).toBe("mittel");
    write("size", "gross", store);
    expect(readSize(store)).toBe("gross");
  });

  it("steps the size with Strg and +, − and 0", () => {
    expect(sizeStep("mittel", "+")).toBe("gross");
    expect(sizeStep("gross", "+")).toBe("gross");
    expect(sizeStep("klein", "-")).toBe("klein");
    expect(sizeStep("klein", "0")).toBe("mittel");
  });
});
