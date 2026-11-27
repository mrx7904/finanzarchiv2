// #55: a page at 100 % in a box lower than the page, over a simulated second. The layout is modelled at its worst:
// the scroll bar takes its room as soon as the page is higher than the box, and the box is empty while a new width
// loads, as it was in 0.7.0. The widths asked and the calls of the core are the real ones of the window, a request is a
// new width the page asks for, a call one that reaches the core.
import { beforeEach, describe, expect, it } from "vitest";
import { forgetPages, loadPage } from "./pageCache";
import { requestWidth } from "./pages";

const BOX = 776;
const SCROLL_BAR = 15;
const ANSWER_MS = 20;

async function simulateSecond() {
  let calls = 0;
  let requests = 0;
  const ask = (document: number, page: number, width: number) => {
    calls += 1;
    return new Promise<{ status: "ok"; data: string }>((resolve) => setTimeout(() => resolve({ status: "ok", data: `page ${document}/${page}@${width}` }), ANSWER_MS));
  };
  let shown: string | null = null;
  let asked: number | null = null;
  const started = Date.now();
  while (Date.now() - started < 1000) {
    // The page is higher than the box once it is shown, so the scroll bar narrows the column.
    const column = BOX - (shown ? SCROLL_BAR : 0);
    const width = requestWidth(column, 100, 1);
    if (width !== asked) {
      asked = width;
      requests += 1;
      shown = null;
      void loadPage(2, 1, width, ask).then((source) => {
        if (asked === width) shown = source;
      });
    }
    // One frame: the column is measured again.
    await new Promise((resolve) => setTimeout(resolve, 16));
  }
  return { calls, requests };
}

describe("a page stands still (#55)", () => {
  beforeEach(() => forgetPages());

  it("asks the core at most twice in a second when the scroll bar comes and goes", async () => {
    const { calls, requests } = await simulateSecond();
    expect(requests).toBeLessThanOrEqual(2);
    expect(calls).toBeLessThanOrEqual(2);
  });

  it("asks only for the widths the core renders", () => {
    expect(requestWidth(761, 100, 1)).toBe(requestWidth(776, 100, 1));
    expect(requestWidth(700, "fit", 1.5)).toBe(1280);
    expect(requestWidth(4000, 300, 2)).toBe(2560);
    expect(requestWidth(40, "fit", 1)).toBe(160);
  });

  it("asks once for a page and width shown twice, and again after a failure", async () => {
    let calls = 0;
    const ok = () => {
      calls += 1;
      return Promise.resolve({ status: "ok" as const, data: "page" });
    };
    await Promise.all([loadPage(1, 1, 640, ok), loadPage(1, 1, 640, ok)]);
    expect(calls).toBe(1);
    const failing = () => {
      calls += 1;
      return Promise.resolve({ status: "error" as const, error: { kind: "internal" } });
    };
    expect(await loadPage(1, 2, 640, failing)).toBeNull();
    expect(await loadPage(1, 2, 640, failing)).toBeNull();
    expect(calls).toBe(3);
  });
});
