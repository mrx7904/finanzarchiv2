import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { colorIn, contrast, luminance, readBlock, type Scheme, type Tokens } from "./contrast";

// Where each text color stands. A new text or surface token gets its pairs here, the last test refuses one without.
const SURFACES = ["--canvas", "--sidebar", "--surface", "--raised", "--field", "--hover", "--selected"];
const PAIRS: [text: string, backgrounds: string[]][] = [
  ["--fg", SURFACES],
  ["--fg-muted", SURFACES],
  ["--fg-accent", ["--canvas", "--sidebar", "--surface", "--raised", "--hover"]],
  ["--on-accent", ["--accent", "--accent-hover"]],
  ["--on-window-close", ["--window-close"]],
  ["--ok", ["--surface", "--raised"]],
  ["--warn", ["--surface", "--raised"]],
  ["--danger", ["--danger-soft", "--surface", "--raised", "--hover", "--selected"]],
];
const TEXT_TOKENS = PAIRS.map(([text]) => text);
const BACKGROUND_TOKENS = [...new Set(PAIRS.flatMap(([, backgrounds]) => backgrounds))];
const SCHEMES: Scheme[] = ["light", "dark"];

const tokensCss = readFileSync("src/ui/tokens.css", "utf8");
const root = readBlock(tokensCss, ":root");

describe("contrast", () => {
  it("computes the ratio of WCAG", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#ffffff")).toBeCloseTo(4.48, 2);
    expect(luminance("#fff")).toBe(1);
  });

  it("reads the right half of light-dark()", () => {
    const tokens: Tokens = new Map([["--x", "light-dark(#111111, #eeeeee)"]]);
    expect(colorIn(tokens, "--x", "light")).toBe("#111111");
    expect(colorIn(tokens, "--x", "dark")).toBe("#eeeeee");
  });
});

describe("the text of tokens.css", () => {
  for (const scheme of SCHEMES) {
    for (const [text, backgrounds] of PAIRS) {
      for (const background of backgrounds) {
        it(`${text} on ${background} in ${scheme} reaches 4.5 to 1`, () => {
          const ratio = contrast(colorIn(root, text, scheme), colorIn(root, background, scheme));
          expect(ratio, `${ratio.toFixed(2)} to 1`).toBeGreaterThanOrEqual(4.5);
        });
      }
    }
  }
});

describe("the token file", () => {
  it("declares every color of the pairs as an opaque hex color in both schemes", () => {
    for (const name of [...TEXT_TOKENS, ...BACKGROUND_TOKENS]) {
      for (const scheme of SCHEMES) expect(colorIn(root, name, scheme)).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("pairs every text color and every surface it declares", () => {
    const colorTokens = [...tokensCss.matchAll(/--color-([\w-]+):\s*var\((--[\w-]+)\)/g)].map((m) => m[2] ?? "");
    const unpaired = colorTokens.filter(
      (name) =>
        (/^--(fg|on-|ok$|warn$|danger$)/.test(name) && !TEXT_TOKENS.includes(name)) ||
        (/^--(canvas|sidebar|surface|raised|field|hover|selected)$/.test(name) && !BACKGROUND_TOKENS.includes(name)),
    );
    expect(unpaired).toEqual([]);
  });
});
