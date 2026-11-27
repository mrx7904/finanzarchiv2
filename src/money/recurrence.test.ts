import { describe, expect, it } from "vitest";
import { recurringState, recurringSums } from "./recurrence";

const row = (over: Partial<Parameters<typeof recurringState>[0]> = {}) => ({ party: 1, incoming: false, ended: false, change: null, perYearCents: -13188, bookings: [1, 2, 3], ...over });

describe("a row of Wiederkehrend (#52)", () => {
  it("says beendet, doppelt?, gestiegen or läuft", () => {
    expect(recurringState(row({ ended: true }), new Set()).label).toBe("beendet");
    expect(recurringState(row(), new Set([2])).label).toBe("doppelt?");
    expect(recurringState(row({ change: "seit März 2026 2,00 € mehr" }), new Set()).label).toBe("gestiegen");
    expect(recurringState(row({ change: "seit März 2026 2,00 € weniger" }), new Set()).label).toBe("läuft");
    expect(recurringState(row(), new Set()).label).toBe("läuft");
  });

  it("sums the running payments out by month and year", () => {
    const sums = recurringSums([row(), row({ perYearCents: -22032 }), row({ ended: true, perYearCents: -99900 }), row({ incoming: true, perYearCents: 4620000 })]);
    expect(sums).toEqual({ month: 2935, year: 35220, running: 2 });
  });
});
