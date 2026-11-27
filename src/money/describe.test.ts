import { describe, expect, it, vi } from "vitest";
import { accountState, amountText, bankLine, bookingFilter, bookingWho, cardLine, centsOf, formatIsoDay, monthGroups, moneyPath, partyYearLine } from "./describe";

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

describe("Konten and Umsätze as designed (#48)", () => {
  it("reads and writes amounts in whole cents, money in with a +", () => {
    expect(centsOf("1.234,56 €")).toBe(123456);
    expect(centsOf("-0,05 €")).toBe(-5);
    expect(amountText(259440)).toBe("+2.594,40 €");
    expect(amountText(-7800, false)).toBe("78,00 €");
    expect(amountText(7800, false)).toBe("78,00 €");
  });
  it("groups the bookings by month, newest first, with the month's sums", () => {
    const groups = monthGroups([
      { day: "2026-03-28", amount: "2.594,40 €", incoming: true },
      { day: "2026-03-01", amount: "780,00 €", incoming: false },
      { day: "2026-02-12", amount: "34,12 €", incoming: false },
    ]);
    expect(groups.map((g) => [g.title, g.income, g.spending, g.rows.length])).toEqual([
      ["März 2026", 259440, 78000, 2],
      ["Februar 2026", 0, 3412, 1],
    ]);
  });
  it("says how an account stands against its bank", () => {
    expect(accountState({ findings: [], bankBalance: "1,00 €" }).label).toBe("stimmt");
    expect(accountState({ findings: ["Am 01.03. …"], bankBalance: "1,00 €" }).label).toBe("prüfen");
    expect(accountState({ findings: [], bankBalance: null }).label).toBe("ohne Bank");
  });
});
