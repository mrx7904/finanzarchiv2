import { describe, expect, it } from "vitest";
import { downloadsLine, formatDay, mailboxLine, kindLabel, textStateLabel, filingLine, stateLabel } from "./describe";

describe("the list of documents", () => {
  it("names every kind in German", () => {
    expect(kindLabel("invoice")).toBe("Rechnung");
    expect(kindLabel("payslip")).toBe("Gehaltsabrechnung");
    expect(kindLabel("receipt")).toBe("Zahlungsbeleg");
    expect(kindLabel("statement")).toBe("Kontoauszug");
    expect(kindLabel("other")).toBe("Sonstiges");
    expect(kindLabel("unknown")).toBe("Art offen");
    expect(kindLabel("something new")).toBe("Art offen");
  });

  it("shows the day of import in German order", () => {
    expect(formatDay("2026-03-05T12:00:00Z")).toBe("05.03.2026");
  });

  it("says what the intake found in the file", () => {
    expect(textStateLabel("text", 3)).toBe("3 Seiten, Text erkannt");
    expect(textStateLabel("text", 1)).toBe("1 Seite, Text erkannt");
    expect(textStateLabel("scan", 2)).toBe("2 Seiten, Scan ohne Text, wird als Bild gelesen");
    expect(textStateLabel("image", null)).toBe("Bild, wird als Bild gelesen");
    expect(textStateLabel(null, 1)).toBe("1 Seite, noch nicht gelesen");
  });
});

describe("the filing of #16", () => {
  it("says where a document stands", () => {
    expect(stateLabel("filed")).toBe("abgelegt");
    expect(stateLabel("waiting")).toBe("wartet auf dich");
    expect(stateLabel("open")).toBe("noch nicht geprüft");
    expect(stateLabel("held")).toBe("zurückgehalten");
    expect(textStateLabel("table", null)).toBe("Tabelle (CSV-Export der Bank)");
  });

  it("names the rule that filed it and when", () => {
    expect(filingLine("filed", "file.checks-passed:1", "2026-11-26T08:00:00Z")).toBe("Von selbst abgelegt am 26.11.2026, alle Prüfungen bestanden.");
    expect(filingLine("filed", "file.statement:1", "2026-11-26T08:00:00Z")).toBe("Als Kontoauszug abgelegt am 26.11.2026.");
    expect(filingLine("waiting", null, null)).toBe("Wartet auf dich, weil:");
  });
});

describe("the downloads of #28", () => {
  it("says what they gave", () => {
    expect(downloadsLine({ on: false, taken: 0, leftOut: 0, lying: 0 })).toMatch(/^Aus./);
    expect(downloadsLine({ on: true, taken: 3, leftOut: 12, lying: 40 })).toBe("Ein: 3 übernommen, 12 ausgelassen, 40 lagen schon da.");
    expect(downloadsLine({ on: true, taken: 1, leftOut: 0, lying: 0 })).toBe("Ein: 1 übernommen, 0 ausgelassen.");
  });
});

describe("the mailboxes of #29", () => {
  it("says why a login failed, or when it worked", () => {
    expect(mailboxLine({ checkedAt: "2026-11-26T10:00:00Z", problem: "Gmail hat die Anmeldung abgelehnt." })).toBe("Gmail hat die Anmeldung abgelehnt.");
    expect(mailboxLine({ checkedAt: "2026-11-26T10:00:00Z", problem: null })).toBe(`Nur lesend angemeldet, zuletzt am ${formatDay("2026-11-26T10:00:00Z")}.`);
    expect(mailboxLine({ checkedAt: null, problem: null })).toBe("Noch nicht geprüft.");
  });
});
