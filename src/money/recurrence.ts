// What the page "Wiederkehrend" (#52, Recurring.tsx) derives for its rows and its head.

type Row = { party: number; incoming: boolean; ended: boolean; change?: string | null; perYearCents: number; bookings: number[] };

/**
 * The state of a recurring payment: "beendet", "gestiegen" when its amount rose (the word joins rule 11 with #52),
 * "doppelt?" when a savings hint names it among two running payments of one kind (#39), else "läuft".
 */
export function recurringState(row: Row, doubled: Set<number>): { tone: "ok" | "warn" | "danger" | "neutral"; label: string } {
  if (row.ended) return { tone: "neutral", label: "beendet" };
  if (row.bookings.some((booking) => doubled.has(booking))) return { tone: "danger", label: "doppelt?" };
  if (row.change?.endsWith("mehr")) return { tone: "warn", label: "gestiegen" };
  return { tone: "neutral", label: "läuft" };
}

/** What the running payments out cost in a month and in a year, in whole cents. */
export function recurringSums(rows: Row[]): { month: number; year: number; running: number } {
  const running = rows.filter((row) => !row.ended && !row.incoming);
  const year = running.reduce((sum, row) => sum + Math.abs(row.perYearCents), 0);
  return { month: Math.round(year / 12), year, running: running.length };
}
