import type { IntakeOverview_Serialize as IntakeOverview } from "../bindings";

/** The German word for the stage of a job. The stages between storing and filing get their work in stage 2. */
export function stageLabel(stage: string): string {
  switch (stage) {
    case "queued":
      return "wartet";
    case "failed":
      return "nicht angenommen";
    case "waiting":
      return "wartet auf dich";
    case "filed":
      return "abgelegt";
    default:
      return "in Arbeit";
  }
}

/** The line at the top of "Eingang". */
export function intakeLine(overview: IntakeOverview): string {
  const parts: string[] = [];
  if (overview.open > 0) parts.push(`${overview.open} ${overview.open === 1 ? "Datei" : "Dateien"} ${overview.paused ? "angehalten" : "in Arbeit"}`);
  if (overview.addedToday > 0) parts.push(`heute ${overview.addedToday} aufgenommen`);
  if (overview.failed.length > 0) parts.push(`${overview.failed.length} nicht angenommen`);
  if (parts.length === 0) return overview.paused ? "Eingang angehalten." : "Nichts im Eingang.";
  const text = parts.join(", ");
  return `${text.charAt(0).toUpperCase()}${text.slice(1)}.`;
}

/** The answer after a pick, before the queue reports. */
export function queuedLine(files: number): string | null {
  if (files === 0) return null;
  return files === 1 ? "1 Datei ist im Eingang." : `${files} Dateien sind im Eingang.`;
}
