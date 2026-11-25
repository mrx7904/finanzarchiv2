import { describe, expect, it } from "vitest";
import { formatDay, kindLabel, textStateLabel } from "./describe";

describe("the list of documents", () => {
  it("names every kind in German", () => {
    expect(kindLabel("invoice")).toBe("Rechnung");
    expect(kindLabel("payslip")).toBe("Gehaltsabrechnung");
    expect(kindLabel("receipt")).toBe("Zahlungsbeleg");
    expect(kindLabel("statement")).toBe("Kontoauszug");
    expect(kindLabel("other")).toBe("Sonstiges");
    expect(kindLabel("unknown")).toBe("Art offen");
    expect(kindLabel("something new")).toBe("Art offen");
  });

  it("shows the day of import in German order", () => {
    expect(formatDay("2026-03-05T12:00:00Z")).toBe("05.03.2026");
  });

  it("says what the intake found in the file", () => {
    expect(textStateLabel("text", 3)).toBe("3 Seiten, Text erkannt");
    expect(textStateLabel("text", 1)).toBe("1 Seite, Text erkannt");
    expect(textStateLabel("scan", 2)).toBe("2 Seiten, Scan ohne Text, wird als Bild gelesen");
    expect(textStateLabel("image", null)).toBe("Bild, wird als Bild gelesen");
    expect(textStateLabel(null, 1)).toBe("1 Seite, noch nicht gelesen");
  });
});
