import { describe, expect, it } from "vitest";
import { searchEntries } from "../App";
import { matchScore } from "../ui/search";
import { applyTheme, chooseTheme, storedTheme } from "./theme";

describe("the pages of Einstellungen", () => {
  const entries = searchEntries({ findings: null, go: () => undefined, openDocument: () => undefined, openBookings: () => undefined, openSettings: () => undefined, back: null });
  const found = (query: string) =>
    entries
      .filter((entry) => matchScore(entry.id, query, [entry.label, ...(entry.detail ? [entry.detail] : []), ...(entry.keywords ?? [])]) > 0)
      .map((entry) => entry.id);

  it("are each found by the quick search in the owner's words (rule 22)", () => {
    expect(found("Gmail")).toContain("settings:postfaecher");
    expect(found("App-Passwort")).toContain("settings:postfaecher");
    expect(found("Downloads")).toContain("settings:ordner");
    expect(found("Namensvorlagen")).toContain("settings:namen");
    expect(found("Kategorie")).toContain("settings:kategorien");
    expect(found("Ollama")).toContain("settings:modell");
    expect(found("Sicherungen")).toContain("settings:archiv");
    expect(found("dunkel")).toContain("settings:darstellung");
    expect(found("Lizenzen")).toContain("settings:ueber");
  });
});

describe("the color scheme", () => {
  it("is kept over a restart, and as the system is no value at all", () => {
    const store = new Map<string, string>();
    const storage = {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => void store.set(key, value),
      removeItem: (key: string) => void store.delete(key),
    };
    const root = { dataset: {} as Record<string, string> } as unknown as HTMLElement;
    expect(storedTheme(storage)).toBe("system");
    globalThis.document = { documentElement: root } as unknown as Document;
    chooseTheme("dark", storage);
    expect(storedTheme(storage)).toBe("dark");
    expect(root.dataset.theme).toBe("dark");
    chooseTheme("system", storage);
    expect(storedTheme(storage)).toBe("system");
    applyTheme("system", root);
    expect(root.dataset.theme).toBeUndefined();
  });
});
