import { describe, expect, it, vi } from "vitest";
import { bankLine, bookingFilter, bookingWho, cardLine, formatIsoDay, moneyPath, partyYearLine } from "./describe";

describe("Konten and Umsätze", () => {
  it("says the bank's last balance with its day", () => {
    expect(formatIsoDay("2026-11-26")).toBe("26.11.2026");
    expect(bankLine("1.234,56 €", "2026-10-31")).toBe("Laut Bank am 31.10.2026: 1.234,56 €");
    expect(bankLine(null, null)).toBe("Noch kein Kontoauszug gelesen");
  });

  it("names a booking by its party, else the bank's other side, else its kind", () => {
    expect(bookingWho("Testhaus Energie", "TESTHAUS ENERGIE AG", "Lastschrift")).toBe("Testhaus Energie");
    expect(bookingWho(null, "TESTHAUS ENERGIE AG", "Lastschrift")).toBe("TESTHAUS ENERGIE AG");
    expect(bookingWho(null, null, "Abschluss")).toBe("Abschluss");
  });

  it("leaves empty fields out of the filter", () => {
    expect(bookingFilter(3, "", "2026-02-28", "", "  ", 200)).toEqual({ account: 3, from: null, to: "2026-02-28", category: null, text: null, offset: 200, limit: 200 });
    expect(bookingFilter(null, "2026-01-01", "", "7", "Miete", 0)).toMatchObject({ account: null, from: "2026-01-01", category: 7, text: "Miete" });
  });

  it("builds the path Konten › Konto › Umsätze with each earlier segment a way back", () => {
    const back = { toAccounts: vi.fn() };
    expect(moneyPath(null, false, back)).toEqual([{ label: "Konten", onSelect: undefined }]);
    const path = moneyPath("Beispielbank …3000", true, back);
    expect(path.map((segment) => segment.label)).toEqual(["Konten", "Beispielbank …3000", "Umsätze"]);
    path[0]?.onSelect?.();
    expect(back.toAccounts).toHaveBeenCalledOnce();
    expect(moneyPath(null, true, back).map((segment) => segment.label)).toEqual(["Konten", "Alle Umsätze"]);
  });
});

describe("an opened booking", () => {
  it("names the party over the year and what a card holds", () => {
    expect(partyYearLine({ party: "Testhaus Energie", year: 2026, count: 4, sum: "254,00 €", incoming: false })).toBe(
      "Testhaus Energie in 2026: 4 Umsätze, zusammen 254,00 € aus",
    );
    expect(partyYearLine({ party: "Beispiel GmbH", year: 2026, count: 1, sum: "3.000,00 €", incoming: true })).toBe(
      "Beispiel GmbH in 2026: 1 Umsatz, zusammen 3.000,00 € ein",
    );
    expect(cardLine("Rechnung", "R-7", "2026-02-01", "85,00 €")).toBe("Rechnung · Nr. R-7 · 01.02.2026 · 85,00 €");
    expect(cardLine("Zahlungsbeleg", null, undefined, "12,00 €")).toBe("Zahlungsbeleg · 12,00 €");
  });
});
