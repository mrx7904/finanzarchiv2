import type { ImportSummary_Serialize as ImportSummary } from "../bindings";

/** One German line about an import, and the refused files with their reasons below it. */
export function describeImport(summary: ImportSummary): { text: string; refused: string[] } {
  if (summary.problem) return { text: summary.problem, refused: [] };
  const parts: string[] = [];
  if (summary.added > 0) parts.push(`${summary.added} ${summary.added === 1 ? "Dokument" : "Dokumente"} aufgenommen`);
  if (summary.known > 0) parts.push(`${summary.known} schon im Archiv`);
  if (summary.refused.length > 0) parts.push(`${summary.refused.length} nicht angenommen`);
  return {
    text: parts.length === 0 ? "Keine Datei gewählt." : `${parts.join(", ")}.`,
    refused: summary.refused.map((file) => `${file.name}: ${file.reason}`),
  };
}

/** The German label of a kind of document. Until stage 2 reads them, every kind is unknown. */
export function kindLabel(kind: string): string {
  return kind === "unknown" ? "unbekannt" : kind;
}

/** "25.11.2026", in the time zone of the computer. */
export function formatDay(iso: string): string {
  return new Intl.DateTimeFormat("de-DE", { day: "2-digit", month: "2-digit", year: "numeric" }).format(new Date(iso));
}
