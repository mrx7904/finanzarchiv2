/** The German label of a kind of document (#10). */
export function kindLabel(kind: string): string {
  const labels: Record<string, string> = {
    invoice: "Rechnung",
    payslip: "Gehaltsabrechnung",
    receipt: "Zahlungsbeleg",
    statement: "Kontoauszug",
    other: "Sonstiges",
    unknown: "Art offen",
  };
  return labels[kind] ?? "Art offen";
}

/** "25.11.2026", in the time zone of the computer. */
export function formatDay(iso: string): string {
  return new Intl.DateTimeFormat("de-DE", { day: "2-digit", month: "2-digit", year: "numeric" }).format(new Date(iso));
}

/** What the intake found in the file: text, a scan without text, or an image. */
export function textStateLabel(state: string | null, pages: number | null): string {
  const pageCount = pages === null ? "" : pages === 1 ? "1 Seite, " : `${pages} Seiten, `;
  switch (state) {
    case "text":
      return `${pageCount}Text erkannt`;
    case "scan":
      return `${pageCount}Scan ohne Text, wird als Bild gelesen`;
    case "image":
      return "Bild, wird als Bild gelesen";
    case "table":
      return "Tabelle (CSV-Export der Bank)";
    default:
      return `${pageCount}noch nicht gelesen`;
  }
}

/** Where a document stands after the checks of #16. */
export function stateLabel(state: string): string {
  switch (state) {
    case "filed":
      return "abgelegt";
    case "waiting":
      return "wartet auf dich";
    case "held":
      return "zurückgehalten";
    default:
      return "noch nicht geprüft";
  }
}

/** The line above "Warum?": who filed it and when, or why it waits. */
export function filingLine(state: string, rule: string | null, decidedAt: string | null): string {
  if (state === "filed" && rule !== null && decidedAt !== null) {
    return rule.startsWith("file.statement") ? `Als Kontoauszug abgelegt am ${formatDay(decidedAt)}.` : `Von selbst abgelegt am ${formatDay(decidedAt)}, alle Prüfungen bestanden.`;
  }
  if (state === "filed") return "Abgelegt.";
  if (state === "waiting") return "Wartet auf dich, weil:";
  if (state === "held") return "Von dir zurückgehalten, es wird nicht von selbst abgelegt.";
  return "Noch nicht geprüft.";
}
