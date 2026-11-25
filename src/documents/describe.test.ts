import { describe, expect, it } from "vitest";
import { describeImport, formatDay, kindLabel } from "./describe";

const empty = { added: 0, known: 0, refused: [], problem: null };

describe("the line after an import", () => {
  it("counts what was taken, known and refused", () => {
    const line = describeImport({
      added: 2,
      known: 1,
      refused: [{ name: "notiz.txt", reason: "Dateien dieser Art nimmt das Archiv nicht an." }],
      problem: null,
    });
    expect(line.text).toBe("2 Dokumente aufgenommen, 1 schon im Archiv, 1 nicht angenommen.");
    expect(line.refused).toEqual(["notiz.txt: Dateien dieser Art nimmt das Archiv nicht an."]);
  });

  it("uses the singular for one document", () => {
    expect(describeImport({ ...empty, added: 1 }).text).toBe("1 Dokument aufgenommen.");
  });

  it("says when nothing was picked", () => {
    expect(describeImport(empty).text).toBe("Keine Datei gewählt.");
  });

  it("shows the problem of an archive that is not ready instead of counts", () => {
    expect(describeImport({ ...empty, problem: "Das Archiv ist nicht bereit." }).text).toBe("Das Archiv ist nicht bereit.");
  });
});

describe("the list of documents", () => {
  it("names an unread kind in German", () => {
    expect(kindLabel("unknown")).toBe("unbekannt");
  });

  it("shows the day of import in German order", () => {
    expect(formatDay("2026-03-05T12:00:00Z")).toBe("05.03.2026");
  });
});
