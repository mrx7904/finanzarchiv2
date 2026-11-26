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
