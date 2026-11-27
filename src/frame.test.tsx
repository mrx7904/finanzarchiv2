// What the frame of #44 gives a view.
import { readFileSync } from "node:fs";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { searchEntries } from "./App";
import { AppShell } from "./ui";

const app = readFileSync("src/App.tsx", "utf8");

describe("the frame", () => {
  it("lists only the views that are built and no line about a healthy archive", () => {
    expect(app).not.toContain("Das Archiv ist bereit");
    expect(app).not.toContain("<Intake />");
    const views = searchEntries({ findings: null, go: () => undefined, openDocument: () => undefined, openBookings: () => undefined, openSettings: () => undefined, back: null })
      .filter((entry) => entry.group === "Ansichten")
      .map((entry) => entry.label);
    expect(views).toEqual(["Prüfen", "Dokumente", "Konten", "Auswertungen", "Einstellungen"]);
  });

  it("shows the arrow back only while there is a way back", () => {
    const without = renderToStaticMarkup(<AppShell path={[{ label: "Dokumente" }]} system="mac">x</AppShell>);
    expect(without).not.toContain('aria-label="Zurück"');
    const withBack = renderToStaticMarkup(
      <AppShell path={[{ label: "Dokumente" }]} system="mac" onBack={() => undefined}>
        x
      </AppShell>,
    );
    expect(withBack).toContain('aria-label="Zurück"');
  });

  it("finds documents, accounts, parties and bookings and opens each in its view", () => {
    const openDocument = vi.fn();
    const openBookings = vi.fn();
    const entries = searchEntries({
      findings: {
        documents: [{ id: 4, name: "Rechnung Beispiel Musik", originalName: "r.pdf" } as never],
        accounts: [{ id: 2, name: "Girokonto" }],
        parties: ["Beispiel Markt"],
        bookings: [{ id: 9, party: "Beispiel Markt", detail: "03.11.2026 · 34,12 €" }],
      },
      go: () => undefined,
      openDocument,
      openBookings,
      openSettings: () => undefined,
      back: () => undefined,
    });
    expect(entries.map((entry) => entry.group)).toEqual(expect.arrayContaining(["Ansichten", "Aktionen", "Dokumente", "Konten", "Partner", "Buchungen"]));
    entries.find((entry) => entry.id === "document:4")?.onSelect();
    expect(openDocument).toHaveBeenCalledWith(4);
    entries.find((entry) => entry.id === "party:Beispiel Markt")?.onSelect();
    expect(openBookings).toHaveBeenCalledWith(null, "Beispiel Markt");
    entries.find((entry) => entry.id === "account:2")?.onSelect();
    expect(openBookings).toHaveBeenCalledWith({ id: 2, name: "Girokonto" }, "");
    expect(entries.some((entry) => entry.id === "action:back")).toBe(true);
  });
});
