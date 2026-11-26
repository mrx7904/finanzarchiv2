import { describe, expect, it } from "vitest";
import { downloadsLine, formatDay, mailParts, mailboxLine, ordersLine, kindLabel, textStateLabel, filingLine, stateLabel } from "./describe";

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
    const none = { readAt: null, looked: 0, taken: 0 };
    expect(mailboxLine({ ...none, checkedAt: "2026-11-26T10:00:00Z", problem: "Gmail hat die Anmeldung abgelehnt." })).toBe("Gmail hat die Anmeldung abgelehnt.");
    expect(mailboxLine({ ...none, checkedAt: "2026-11-26T10:00:00Z", problem: null })).toBe("Nur lesend angemeldet, wird gleich gelesen.");
    expect(mailboxLine({ ...none, checkedAt: null, problem: null })).toBe("Noch nicht geprüft.");
    expect(mailboxLine({ checkedAt: null, problem: null, readAt: "2026-11-26T10:00:00Z", looked: 412, taken: 9 })).toBe(
      `Nur lesend, zuletzt gelesen am ${formatDay("2026-11-26T10:00:00Z")}: 412 Mails angesehen, 9 mit Beleg.`,
    );
    expect(mailboxLine({ checkedAt: null, problem: null, readAt: "2026-11-26T10:00:00Z", looked: 1, taken: 0 })).toContain("1 Mail angesehen");
  });
});

describe("a mail kept as its text, #31", () => {
  it("splits its head from its text", () => {
    const { head, body } = mailParts("Von: Beispiel GmbH <info@example.com>\nBetreff: Ihre Rechnung: R-7\n\nGesamt 23,80 €\nDanke\n");
    expect(head).toEqual([
      ["Von", "Beispiel GmbH <info@example.com>"],
      ["Betreff", "Ihre Rechnung: R-7"],
    ]);
    expect(body).toBe("Gesamt 23,80 €\nDanke");
    expect(textStateLabel("mail", null)).toBe("Text einer Mail");
  });
});

describe("the orders of #32", () => {
  it("says how many wait for their invoice", () => {
    expect(ordersLine([])).toMatch(/^Bestellungen, die/);
    expect(ordersLine([{ invoice: 3 }, { invoice: 4 }])).toBe("Jede der 2 Bestellungen hat ihre Rechnung.");
    expect(ordersLine([{ invoice: 3 }, { invoice: null }])).toMatch(/^1 Bestellung wartet auf die Rechnung\./);
    expect(ordersLine([{}, { invoice: null }])).toMatch(/^2 Bestellungen warten/);
  });
});
