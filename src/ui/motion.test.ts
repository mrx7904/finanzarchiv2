import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { MotionGlobalConfig } from "motion/react";
import { afterEach, describe, expect, it } from "vitest";
import { applyReducedMotion, followReducedMotion } from "./motion";

/** Every source file of the UI with its text, tests left out. */
function sources(dir = "src"): [file: string, text: string][] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry): [string, string][] => {
    const full = path.join(dir, entry.name).split(path.sep).join("/");
    if (entry.isDirectory()) return sources(full);
    if (!/\.(tsx?|css)$/.test(entry.name) || /\.test\.tsx?$/.test(entry.name)) return [];
    return [[full, readFileSync(full, "utf8")]];
  });
}

/** A media query list whose answer the test changes. */
function fakeQuery(matches: boolean) {
  const listeners = new Set<() => void>();
  return {
    matches,
    addEventListener: (_type: string, listener: () => void) => listeners.add(listener),
    removeEventListener: (_type: string, listener: () => void) => listeners.delete(listener),
    change(to: boolean) {
      this.matches = to;
      for (const listener of listeners) listener();
    },
    get listening() {
      return listeners.size;
    },
  };
}

afterEach(() => applyReducedMotion(false));

describe("reduced motion stops Motion", () => {
  it("skips every animation of Motion while the system asks for reduced motion", () => {
    const query = fakeQuery(true);
    followReducedMotion(query as unknown as MediaQueryList);
    expect(MotionGlobalConfig.skipAnimations).toBe(true);
  });

  it("follows the setting when it changes while the app runs, and stops listening on cleanup", () => {
    const query = fakeQuery(false);
    const stop = followReducedMotion(query as unknown as MediaQueryList);
    expect(MotionGlobalConfig.skipAnimations).toBe(false);
    query.change(true);
    expect(MotionGlobalConfig.skipAnimations).toBe(true);
    query.change(false);
    expect(MotionGlobalConfig.skipAnimations).toBe(false);
    stop();
    expect(query.listening).toBe(0);
  });

  it("is the only way into Motion, so no piece can animate around the gate", () => {
    const direct = sources().filter(([file, text]) => file !== "src/ui/motion.tsx" && /from\s+["'](motion|motion\/react|framer-motion)(\/[^"']*)?["']/.test(text));
    expect(direct.map(([file]) => file)).toEqual([]);
  });

  it("is mounted around the app", () => {
    expect(readFileSync("src/main.tsx", "utf8")).toMatch(/<MotionRoot>\s*<App \/>\s*<\/MotionRoot>/);
  });

  it("is not switched off by a piece", () => {
    const offenders = sources().filter(([, text]) => /reducedMotion=["']never["']|respectMotionPreference=\{false\}|\.animate\(/.test(text));
    expect(offenders.map(([file]) => file)).toEqual([]);
  });
});

describe("reduced motion stops every CSS transition and animation", () => {
  const tokens = readFileSync("src/ui/tokens.css", "utf8");
  const reduced = /@media \(prefers-reduced-motion: reduce\)\s*\{([\s\S]*?)\n\}/.exec(tokens)?.[1] ?? "";

  it("sets every duration token to zero", () => {
    const durations = [...tokens.matchAll(/(--duration-[\w-]+):/g)].map((m) => m[1]);
    expect(durations.length).toBeGreaterThan(0);
    for (const name of new Set(durations)) expect(reduced).toMatch(new RegExp(`${name}:\\s*0s;`));
  });

  it("overrides with !important what a piece sets inline or with a class", () => {
    expect(reduced).toMatch(/\*,\s*\*::before,\s*\*::after\s*\{/);
    for (const property of ["transition-duration", "transition-delay", "animation-duration", "animation-delay"]) {
      expect(reduced).toMatch(new RegExp(`${property}:\\s*0s !important;`));
    }
    expect(reduced).toMatch(/animation-iteration-count:\s*1 !important;/);
  });

  it("times the transitions of the stylesheets with the duration tokens", () => {
    const literal = sources()
      .filter(([file]) => file.endsWith(".css"))
      .flatMap(([file, text]) =>
        [...text.replace(/@media \(prefers-reduced-motion: reduce\)[\s\S]*?\n\}/, "").matchAll(/(transition|animation)(-duration)?\s*:[^;]*?\b\d+(\.\d+)?m?s\b[^;]*;/g)].map((m) => `${file}: ${m[0]}`),
      );
    expect(literal).toEqual([]);
  });
});
