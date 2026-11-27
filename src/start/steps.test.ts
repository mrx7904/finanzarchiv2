import { describe, expect, it } from "vitest";
import { nextSteps, startSentence } from "./steps";

const quiet = { waiting: 0, modelProblem: null, folderProblems: 0, mailboxProblems: 0, gaps: [], ordersMissing: 0, hints: [] };

describe("the next steps of Start (#40)", () => {
  it("are none for a quiet archive", () => {
    expect(nextSteps(quiet)).toEqual([]);
    expect(startSentence([])).toBe("Nichts wartet.");
  });

  it("put the most pressing first, each with one action and its place", () => {
    const steps = nextSteps({
      ...quiet,
      waiting: 3,
      ordersMissing: 2,
      modelProblem: "Ollama antwortet nicht.",
      gaps: [{ id: 4, name: "Girokonto" }],
      mailboxProblems: 1,
      hints: [{ key: "fees:2026", subject: "Bankgebühren", line: "2026: 8,45 € in 2 Buchungen" }],
    });
    expect(steps.map((step) => step.key)).toEqual(["model", "mailboxes", "review", "gap:4", "orders", "hint:fees:2026"]);
    expect(steps[2]).toMatchObject({ title: "3 Fragen warten in Prüfen", action: "Prüfen", target: { view: "review" } });
    expect(steps[3]?.target).toEqual({ view: "accounts", account: { id: 4, name: "Girokonto" } });
    expect(startSentence(steps)).toBe("6 Dinge warten.");
  });
});
