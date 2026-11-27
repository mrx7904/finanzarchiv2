import { describe, expect, it } from "vitest";
import { balancesOfYear, lastMonth, monthRange, monthValues, nettoByMonth, periodWords, sameMonths, savingsRate } from "./derive";

const sums = (cents: number[]) => Array.from({ length: 12 }, (_, at) => ({ cents: cents[at] ?? 0, count: cents[at] ? 1 : 0 }));

describe("Auswertungen (#38)", () => {
  it("stops the bars after the last month with a booking and compares the same months of the year before", () => {
    const income = sums([100, 100, 100]);
    const spending = sums([50, 0, 70]);
    expect(lastMonth(income, spending)).toBe(2);
    expect(monthValues(spending, 2)).toEqual([50, 0, 70, null, null, null, null, null, null, null, null, null]);
    expect(sameMonths(sums([10, 20, 30, 40, 50]), 2)).toBe(60);
    expect(lastMonth(sums([]), sums([]))).toBe(-1);
  });

  it("names the savings rate and the period", () => {
    expect(savingsRate(2613860, 1674640)).toBe(36);
    expect(savingsRate(0, 100)).toBeNull();
    expect(periodWords(9)).toBe("Januar bis Oktober");
    expect(periodWords(0)).toBe("Januar");
  });

  it("gives a month to Umsätze as its first and last day", () => {
    expect(monthRange(2026, 1)).toEqual({ from: "2026-02-01", to: "2026-02-28", label: "Februar 2026" });
    expect(monthRange(2024, 1).to).toBe("2024-02-29");
  });

  it("shows a month without a payslip as missing and the balances of the year", () => {
    const netto = nettoByMonth([
      { month: "2026-01", missing: false, amounts: [{ field: "netto", cents: 259440 }] },
      { month: "2026-02", missing: true, amounts: [] },
    ]);
    expect(netto.slice(0, 3)).toEqual([259440, null, undefined]);
    expect(balancesOfYear([{ day: "2025-12-31", cents: 1 }, { day: "2026-01-31", cents: 2 }], 2026)).toEqual({ values: [2], labels: ["31.01."] });
  });
});
