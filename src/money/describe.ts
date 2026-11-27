import type { BookingFilter } from "../bindings";

/** "25.11.2026" from an ISO day, without the time zone moving it. */
export function formatIsoDay(iso: string): string {
  const [year, month, day] = iso.split("-");
  return year && month && day ? `${day}.${month}.${year}` : iso;
}

/** The line under an account: the bank's last balance with its day, or that none was read. */
export function bankLine(bankBalance: string | null | undefined, bankDay: string | null | undefined): string {
  if (bankBalance == null || bankDay == null) return "Noch kein Kontoauszug gelesen";
  return `Laut Bank am ${formatIsoDay(bankDay)}: ${bankBalance}`;
}

/** Who a booking is with: its party, else the other side the bank printed, else its kind. */
export function bookingWho(party: string | null | undefined, otherSide: string | null | undefined, kind: string): string {
  return party ?? otherSide ?? kind;
}

/** The filter of "Umsätze" from the fields of the view, empty fields left out. */
export function bookingFilter(account: number | null, from: string, to: string, category: string, text: string, offset: number): BookingFilter {
  return {
    account,
    from: from === "" ? null : from,
    to: to === "" ? null : to,
    category: category === "" ? null : Number(category),
    text: text.trim() === "" ? null : text.trim(),
    offset,
    limit: 200,
  };
}

/** The path in the top bar: "Konten › <Konto> › Umsätze", each earlier segment leads back. */
export function moneyPath(account: string | null, bookings: boolean, back: { toAccounts: () => void }) {
  const path: { label: string; onSelect?: () => void }[] = [{ label: "Konten", onSelect: bookings ? back.toAccounts : undefined }];
  if (bookings) {
    if (account) path.push({ label: account, onSelect: back.toAccounts });
    path.push({ label: account ? "Umsätze" : "Alle Umsätze" });
  }
  return path;
}

/** The heading of a party's year in an opened booking: "Testhaus Energie in 2026: 4 Umsätze, zusammen 254,00 € aus". */
export function partyYearLine(year: { party: string; year: number; count: number; sum: string; incoming: boolean }): string {
  const bookings = year.count === 1 ? "1 Umsatz" : `${year.count} Umsätze`;
  return `${year.party} in ${year.year}: ${bookings}, zusammen ${year.sum} ${year.incoming ? "ein" : "aus"}`;
}

/** The line of a document card: kind, number, day, amount, whichever were read. */
export function cardLine(kind: string, number: string | null | undefined, day: string | null | undefined, amount: string | null | undefined): string {
  return [kind, number ? `Nr. ${number}` : null, day ? formatIsoDay(day) : null, amount].filter(Boolean).join(" · ");
}

/** Whole cents of an amount as the core writes it ("1.234,56 €", "-0,05 €"), so sums stay exact (#48). */
export function centsOf(text: string): number {
  const negative = text.trim().startsWith("-");
  const digits = text.replace(/[^\d]/g, "");
  return (negative ? -1 : 1) * Number(digits || "0");
}

/** Cents as the window shows them, money in with a "+" and money out without a sign (rule 13). */
export function amountText(cents: number, incoming = cents > 0): string {
  const magnitude = Math.abs(cents);
  const euros = Math.floor(magnitude / 100).toLocaleString("de-DE");
  const text = `${euros},${String(magnitude % 100).padStart(2, "0")} €`;
  return incoming ? `+${text}` : text;
}

const MONTHS = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];

/** The bookings grouped by month, newest first, each month with its money in and out. */
export function monthGroups<T extends { day: string; amount: string; incoming: boolean }>(rows: T[]): { key: string; title: string; income: number; spending: number; rows: T[] }[] {
  const groups: { key: string; title: string; income: number; spending: number; rows: T[] }[] = [];
  for (const row of rows) {
    const key = row.day.slice(0, 7);
    let group = groups.find((one) => one.key === key);
    if (!group) {
      group = { key, title: `${MONTHS[Number(key.slice(5, 7)) - 1] ?? ""} ${key.slice(0, 4)}`, income: 0, spending: 0, rows: [] };
      groups.push(group);
    }
    group.rows.push(row);
    const cents = Math.abs(centsOf(row.amount));
    if (row.incoming) group.income += cents;
    else group.spending += cents;
  }
  return groups.sort((a, b) => (a.key < b.key ? 1 : a.key > b.key ? -1 : 0));
}

/** An account against its bank: "stimmt", "prüfen" when something does not match, "ohne Bank" without a balance of it. */
export function accountState(account: { findings: string[]; bankBalance?: string | null }): { tone: "ok" | "warn" | "neutral"; label: string } {
  if (account.findings.length > 0) return { tone: "warn", label: "prüfen" };
  if (!account.bankBalance) return { tone: "neutral", label: "ohne Bank" };
  return { tone: "ok", label: "stimmt" };
}
