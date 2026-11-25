import { describe, expect, it } from "vitest";
import { fold, matchScore } from "./search";

describe("the quick search", () => {
  it("folds case, accents and ß", () => {
    expect(fold("Straße Bücherei ÉTÉ")).toBe("strasse bucherei ete");
  });

  it("finds without umlauts and ß", () => {
    expect(matchScore("d6", "bucherei", ["Bücherei Ahorn"])).toBe(1);
    expect(matchScore("d9", "strasse", ["Hauptstraße 1"])).toBe(0.5);
  });

  it("needs every word and ranks a word start above a hit inside a word", () => {
    expect(matchScore("d1", "stadt werke", ["Stadtwerke Beispielstadt"])).toBe(0.5);
    expect(matchScore("d1", "beispiel stadtwerke", ["Stadtwerke Beispielstadt"])).toBe(1);
    expect(matchScore("d1", "stadtwerke nord", ["Stadtwerke Beispielstadt"])).toBe(0);
  });

  it("does not search the id of an entry", () => {
    expect(matchScore("d1", "d1", ["Stadtwerke Beispielstadt"])).toBe(0);
  });
});
