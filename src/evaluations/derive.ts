// What "Auswertungen" (#38) derives from the rows of the core, kept pure for its tests.

type Sum = { cents: number; count: number };

/** The last month (0 to 11) with a booking, -1 for a year without any. */
export function lastMonth(income: Sum[], spending: Sum[]): number {
  for (let month = 11; month >= 0; month -= 1) if ((income[month]?.count ?? 0) + (spending[month]?.count ?? 0) > 0) return month;
  return -1;
}

/** The months of a year as bar values: a month after the last with a booking is no value at all. */
export function monthValues(sums: Sum[], last: number): (number | null)[] {
  return Array.from({ length: 12 }, (_, month) => (month > last ? null : (sums[month]?.cents ?? 0)));
}

/** The sum of the same months of another year, so a year that is still open compares honestly. */
export function sameMonths(sums: Sum[], last: number): number {
  return sums.slice(0, last + 1).reduce((total, sum) => total + sum.cents, 0);
}

/** The share of the income kept, in whole percent, `null` without income. */
export function savingsRate(income: number, spending: number): number | null {
  return income > 0 ? Math.round(((income - spending) / income) * 100) : null;
}

const MONTHS = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];

/** "Januar bis Oktober", the months a figure covers. */
export function periodWords(last: number): string {
  if (last < 0) return "";
  return last === 0 ? "Januar" : `Januar bis ${MONTHS[last]}`;
}

/** The first and the last day of a month, for the filter of a jump to Umsätze. */
export function monthRange(year: number, month: number): { from: string; to: string; label: string } {
  const days = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
  const at = String(month + 1).padStart(2, "0");
  return { from: `${year}-${at}-01`, to: `${year}-${at}-${days}`, label: `${MONTHS[month]} ${year}` };
}

/** The Netto of each month of the payslips of a year: a missing month in their range is `null`, shown as "fehlt"; a month outside it is `undefined` and shows nothing (#35). */
export function nettoByMonth(months: { month: string; missing: boolean; amounts: { field: string; cents: number }[] }[]): (number | null | undefined)[] {
  const values: (number | null | undefined)[] = Array.from({ length: 12 }, () => undefined);
  for (const row of months) {
    const at = Number(row.month.slice(5, 7)) - 1;
    values[at] = row.missing ? null : (row.amounts.find((amount) => amount.field === "netto")?.cents ?? null);
  }
  return values;
}

/** The balances of all accounts at the end of each month of a year. */
export function balancesOfYear(points: { day: string; cents: number }[], year: number): { values: number[]; labels: string[] } {
  const inYear = points.filter((point) => point.day.startsWith(String(year)));
  return { values: inYear.map((point) => point.cents), labels: inYear.map((point) => `${point.day.slice(8, 10)}.${point.day.slice(5, 7)}.`) };
}
