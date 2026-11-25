import { describe, expect, it } from "vitest";
import { formatDay, kindLabel } from "./describe";

describe("the list of documents", () => {
  it("names an unread kind in German", () => {
    expect(kindLabel("unknown")).toBe("unbekannt");
  });

  it("shows the day of import in German order", () => {
    expect(formatDay("2026-03-05T12:00:00Z")).toBe("05.03.2026");
  });
});
