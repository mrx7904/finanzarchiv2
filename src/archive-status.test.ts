import { describe, expect, it } from "vitest";
import { describeArchive, describeLoaded } from "./archive-status";

const snapshot = { name: "finanzarchiv-20261125T080000Z-taeglich.db", takenAt: "2026-11-25T08:00:00Z" };

describe("the state of the archive", () => {
  it("reads a ready archive as ready", () => {
    expect(describeArchive({ state: "ready" })).toEqual({ tone: "ok", text: "Das Archiv ist bereit." });
  });

  it("never reads a damaged archive as empty and counts its snapshots", () => {
    const line = describeArchive({ state: "damaged", snapshots: [snapshot, snapshot] });
    expect(line.tone).toBe("problem");
    expect(line.text).toContain("beschädigt");
    expect(line.text).toContain("2 Sicherungen");
  });

  it("names a missing file beside one snapshot", () => {
    const line = describeArchive({ state: "missing", snapshots: [snapshot] });
    expect(line.text).toContain("fehlt");
    expect(line.text).toContain("Eine Sicherung");
  });

  it("shows the German message of a failed command and no version", () => {
    const line = describeLoaded({ status: "error", error: { kind: "internal", message: "Bitte neu starten." } });
    expect(line).toEqual({ tone: "problem", text: "Bitte neu starten." });
  });

  it("carries the version of a loaded status", () => {
    const line = describeLoaded({ status: "ok", data: { version: "0.1.0", archive: { state: "ready" } } });
    expect(line.version).toBe("0.1.0");
  });
});
