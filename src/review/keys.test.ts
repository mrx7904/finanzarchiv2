import { describe, expect, it, vi } from "vitest";
import { inSequence, questionLine, reviewKey, reviewPath } from "./keys";

const list = { count: 3, index: 1, open: false, readings: 0 };

describe("the keys of Prüfen", () => {
  it("moves with J and K within the list and opens with Enter", () => {
    expect(reviewKey("j", list)).toEqual({ kind: "move", index: 2 });
    expect(reviewKey("J", { ...list, index: 2 })).toEqual({ kind: "move", index: 2 });
    expect(reviewKey("k", list)).toEqual({ kind: "move", index: 0 });
    expect(reviewKey("k", { ...list, index: 0 })).toEqual({ kind: "move", index: 0 });
    expect(reviewKey("Enter", list)).toEqual({ kind: "open" });
    expect(reviewKey("x", list)).toEqual({ kind: "none" });
    expect(reviewKey("j", { ...list, count: 0 })).toEqual({ kind: "none" });
  });

  it("picks a reading with 1 and 2 when a question is open, and goes back with Escape", () => {
    const open = { ...list, open: true, readings: 2 };
    expect(reviewKey("1", open)).toEqual({ kind: "pick", reading: 0 });
    expect(reviewKey("2", open)).toEqual({ kind: "pick", reading: 1 });
    expect(reviewKey("3", open)).toEqual({ kind: "none" });
    expect(reviewKey("j", open)).toEqual({ kind: "none" });
    expect(reviewKey("Escape", open)).toEqual({ kind: "back" });
  });

  it("builds the path with each earlier segment a way back", () => {
    const back = { toStart: vi.fn(), toGroup: vi.fn() };
    const path = reviewPath("Bruttobetrag ungleich gelesen", "rechnung.pdf", back);
    expect(path.map((segment) => segment.label)).toEqual(["Prüfen", "Bruttobetrag ungleich gelesen", "rechnung.pdf"]);
    path[0]?.onSelect?.();
    path[1]?.onSelect?.();
    expect(back.toStart).toHaveBeenCalledOnce();
    expect(back.toGroup).toHaveBeenCalledOnce();
    expect(path[2]?.onSelect).toBeUndefined();
    expect(reviewPath(null, null, back)).toEqual([{ label: "Prüfen", onSelect: undefined }]);
  });
});

describe("Der Reihe nach prüfen (#47)", () => {
  it("goes through every reason in its order and asks each document once", () => {
    const groups = [{ key: "check:kind" }, { key: "check:agree" }, { key: "link:choose" }];
    const order = inSequence(groups, {
      "check:agree": [{ document: 3 }, { document: 1 }],
      "check:kind": [{ document: 1 }, { document: 2 }],
      "link:choose": [{ document: 4 }],
    });
    expect(order.map((question) => question.document)).toEqual([1, 2, 3, 4]);
    expect(inSequence([], {})).toEqual([]);
  });

  it("names the question it stands at", () => {
    expect(questionLine(0, 12)).toBe("Frage 1 von 12");
  });
});
