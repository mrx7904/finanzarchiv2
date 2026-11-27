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
    case "mail":
      return "Text einer Mail";
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

/** What the downloads gave (#28): off, or how many files were taken and left out. */
export function downloadsLine(state: { on: boolean; taken: number; leftOut: number; lying: number }): string {
  if (!state.on) return "Aus. Eingeschaltet nimmt das Archiv aus den Downloads nur, was wie eine Rechnung, Gehaltsabrechnung, Quittung oder ein Kontoauszug aussieht.";
  const parts = [`${state.taken} übernommen`, `${state.leftOut} ausgelassen`];
  if (state.lying > 0) parts.push(`${state.lying} lagen schon da`);
  return `Ein: ${parts.join(", ")}.`;
}

/** How a mailbox stands (#29, #31): why its last login failed, or what its rounds gave. */
export function mailboxLine(mailbox: { checkedAt: string | null; problem: string | null; readAt: string | null; looked: number; taken: number }): string {
  if (mailbox.problem) return mailbox.problem;
  if (mailbox.readAt) {
    const mails = mailbox.looked === 1 ? "1 Mail" : `${mailbox.looked} Mails`;
    return `Nur lesend, zuletzt gelesen am ${formatDay(mailbox.readAt)}: ${mails} angesehen, ${mailbox.taken} mit Beleg.`;
  }
  if (mailbox.checkedAt) return "Nur lesend angemeldet, wird gleich gelesen.";
  return "Noch nicht geprüft.";
}

/** The head and the text of a mail kept as its text (#31): "Von: …" lines up to the first empty line. */
export function mailParts(text: string): { head: [string, string][]; body: string } {
  const lines = text.split("\n");
  const empty = lines.indexOf("");
  const headLines = empty < 0 ? [] : lines.slice(0, empty);
  const head: [string, string][] = headLines.map((line) => {
    const at = line.indexOf(": ");
    return at < 0 ? ["", line] : [line.slice(0, at), line.slice(at + 2)];
  });
  return { head, body: (empty < 0 ? lines : lines.slice(empty + 1)).join("\n").trim() };
}

/** What "Bestellungen" says on top (#32): how many orders wait for their invoice, and where it goes. */
export function ordersLine(orders: { invoice?: number | null }[]): string {
  const missing = orders.filter((order) => typeof order.invoice !== "number").length;
  if (orders.length === 0) return "Bestellungen, die die Mails von Amazon nennen, mit ihrer Rechnung.";
  if (missing === 0) return `Jede der ${orders.length} Bestellungen hat ihre Rechnung.`;
  const which = missing === 1 ? "1 Bestellung wartet" : `${missing} Bestellungen warten`;
  return `${which} auf die Rechnung. Lade sie bei Amazon unter „Meine Bestellungen“ herunter und lege sie in den Eingangsordner oder die Downloads, das Archiv ordnet sie über die Bestellnummer zu.`;
}

/** The state of a row of Dokumente in the words of rule 11 (#46). */
export function rowState(state: string): { tone: "ok" | "warn" | "danger" | "neutral"; label: string } {
  switch (state) {
    case "filed":
      return { tone: "ok", label: "bestätigt" };
    case "waiting":
    case "held":
      return { tone: "warn", label: "prüfen" };
    default:
      return { tone: "neutral", label: "läuft" };
  }
}

/** The line under the name of a row: its kind and the day it came. */
export function rowDetail(row: { kind: string; importedAt: string }): string {
  return `${kindLabel(row.kind)} · ${formatDay(row.importedAt)}`;
}

/** The line of counts under the filters: how many are shown and how many wait. */
export function countsLine(rows: { state: string }[]): string {
  const waiting = rows.filter((row) => rowState(row.state).label === "prüfen").length;
  const shown = `${rows.length.toLocaleString("de-DE")} ${rows.length === 1 ? "Dokument" : "Dokumente"}`;
  return waiting > 0 ? `${shown} · ${waiting} prüfen` : shown;
}

/** The rows a search, a kind and a state leave. */
export function filterRows<T extends { name: string; originalName: string; kind: string; state: string }>(rows: T[], search: string, kind: string, state: string): T[] {
  const words = search.toLowerCase().split(/\s+/).filter(Boolean);
  return rows.filter(
    (row) =>
      (kind === "" || row.kind === kind) &&
      (state === "" || rowState(row.state).label === state) &&
      words.every((word) => `${row.name} ${row.originalName}`.toLowerCase().includes(word)),
  );
}

/** The state of an order: "vollständig" with an invoice or when it needs none, else "fehlt" (#46). */
export function orderState(order: { invoice: number | null; noInvoice: boolean }): { tone: "ok" | "danger"; label: string } {
  return order.invoice !== null || order.noInvoice ? { tone: "ok", label: "vollständig" } : { tone: "danger", label: "fehlt" };
}

/** The line of counts of Bestellungen. */
export function orderCountsLine(orders: { invoice: number | null; noInvoice: boolean }[]): string {
  const missing = orders.filter((order) => orderState(order).label === "fehlt").length;
  const shown = `${orders.length} ${orders.length === 1 ? "Bestellung" : "Bestellungen"}`;
  return missing > 0 ? `${shown} · ${missing} fehlt` : shown;
}
