import { describe, expect, it } from "vitest";
import { intakeLine, queuedLine, stageLabel } from "./describe";

const overview = { paused: false, open: 0, running: [], failed: [], addedToday: 0 };
const failed = { id: 1, name: "notiz.txt", stage: "failed", error: "Dateien dieser Art nimmt das Archiv nicht an." };

describe("the intake", () => {
  it("names the stages in German", () => {
    expect(stageLabel("queued")).toBe("wartet");
    expect(stageLabel("stored")).toBe("in Arbeit");
    expect(stageLabel("checked")).toBe("in Arbeit");
    expect(stageLabel("failed")).toBe("nicht angenommen");
    expect(stageLabel("model")).toBe("wartet auf das Modell");
  });

  it("says what runs, what came today and what was refused", () => {
    expect(intakeLine({ ...overview, open: 12, addedToday: 40, failed: [failed] })).toBe(
      "12 Dateien in Arbeit, heute 40 aufgenommen, 1 nicht angenommen.",
    );
  });

  it("says when the queue is paused", () => {
    expect(intakeLine({ ...overview, open: 1, paused: true })).toBe("1 Datei angehalten.");
    expect(intakeLine({ ...overview, paused: true })).toBe("Eingang angehalten.");
  });

  it("says when nothing is there", () => {
    expect(intakeLine(overview)).toBe("Nichts im Eingang.");
  });

  it("answers a pick before the workers report", () => {
    expect(queuedLine(0)).toBeNull();
    expect(queuedLine(1)).toBe("1 Datei ist im Eingang.");
    expect(queuedLine(30)).toBe("30 Dateien sind im Eingang.");
  });
});
