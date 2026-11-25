// Invented documents for the showcase of #07. No sender, number or amount here is real (decision-public-repository.md).
import type { Tone } from "../ui";

export type SampleDocument = {
  id: string;
  sender: string;
  kind: string;
  date: string;
  number: string;
  cents: number;
  category: string;
  state: { tone: Tone; label: string };
};

export const documents: SampleDocument[] = [
  { id: "d1", sender: "Stadtwerke Beispielstadt", kind: "Rechnung", date: "14.11.2026", number: "SW-2026-0413", cents: 8_640, category: "wohnen", state: { tone: "warn", label: "prüfen" } },
  { id: "d2", sender: "Telefon Nord GmbH", kind: "Rechnung", date: "12.11.2026", number: "TN 55120", cents: 3_999, category: "kommunikation", state: { tone: "warn", label: "prüfen" } },
  { id: "d3", sender: "Versicherung Muster AG", kind: "Beitragsrechnung", date: "08.11.2026", number: "VM-88-1207", cents: 21_450, category: "versicherung", state: { tone: "danger", label: "doppelt?" } },
  { id: "d4", sender: "Autohaus Lindenweg", kind: "Rechnung", date: "03.11.2026", number: "AL-7781", cents: 48_912, category: "auto", state: { tone: "ok", label: "bestätigt" } },
  { id: "d5", sender: "Praxis am Markt", kind: "Quittung", date: "29.10.2026", number: "Q-1904", cents: 2_500, category: "gesundheit", state: { tone: "ok", label: "bestätigt" } },
  { id: "d6", sender: "Bücherei Ahorn", kind: "Rechnung", date: "21.10.2026", number: "BA 3310", cents: 1_890, category: "bildung", state: { tone: "neutral", label: "gelesen" } },
  { id: "d7", sender: "Musterbank", kind: "Kontoauszug", date: "01.10.2026", number: "Auszug 10/2026", cents: 0, category: "konto", state: { tone: "neutral", label: "gelesen" } },
];

export const categories = [
  { value: "wohnen", label: "Wohnen und Energie" },
  { value: "kommunikation", label: "Telefon und Internet" },
  { value: "versicherung", label: "Versicherungen" },
  { value: "auto", label: "Auto" },
  { value: "gesundheit", label: "Gesundheit" },
  { value: "bildung", label: "Bücher und Bildung" },
  { value: "konto", label: "Konto" },
];

const grouping = new Intl.NumberFormat("de-DE");

/** Display only: the core owns every amount, the UI formats whole cents without a float. */
export function formatCents(cents: number): string {
  const sign = cents < 0 ? "−" : "";
  const whole = Math.abs(cents);
  return `${sign}${grouping.format(Math.floor(whole / 100))},${String(whole % 100).padStart(2, "0")} €`;
}
