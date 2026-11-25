import type { AppStatus, ArchiveStatus, CommandError } from "./bindings";

export type StatusLine = { tone: "ok" | "problem"; text: string };

function snapshotsSentence(count: number): string {
  if (count === 0) return "Es gibt keine Sicherung.";
  return count === 1 ? "Eine Sicherung liegt bereit." : `${count} Sicherungen liegen bereit.`;
}

/** The state of the archive as one German sentence. A damaged archive never reads as an empty one. */
export function describeArchive(archive: ArchiveStatus): StatusLine {
  switch (archive.state) {
    case "ready":
      return { tone: "ok", text: "Das Archiv ist bereit." };
    case "damaged":
      return {
        tone: "problem",
        text: `Die Archivdatei ist beschädigt und bleibt unverändert. ${snapshotsSentence(archive.snapshots.length)}`,
      };
    case "missing":
      return {
        tone: "problem",
        text: `Die Archivdatei fehlt, daneben liegen Sicherungen. ${snapshotsSentence(archive.snapshots.length)}`,
      };
  }
}

export type LoadedStatus = { status: "ok"; data: AppStatus } | { status: "error"; error: CommandError };

export function describeLoaded(loaded: LoadedStatus): StatusLine & { version?: string } {
  if (loaded.status === "error") return { tone: "problem", text: loaded.error.message };
  return { ...describeArchive(loaded.data.archive), version: loaded.data.version };
}
