// The contrast ratio of WCAG 2.2 (w3.org/TR/WCAG22, "contrast ratio" and "relative luminance") and a reader for the
// color tokens of tokens.css, so the test checks the values the app really uses.

export type Scheme = "light" | "dark";
export type Tokens = Map<string, string>;

/** `#rgb` or `#rrggbb` as three channels from 0 to 255. */
export function parseHex(hex: string): [number, number, number] {
  const match = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim());
  if (!match?.[1]) throw new Error(`not an opaque hex color: ${hex}`);
  const digits = match[1].length === 3 ? [...match[1]].map((d) => d + d).join("") : match[1];
  return [0, 2, 4].map((i) => Number.parseInt(digits.slice(i, i + 2), 16)) as [number, number, number];
}

function channel(value: number): number {
  const c = value / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

export function luminance(hex: string): number {
  const [r, g, b] = parseHex(hex);
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/** The custom properties declared directly in each block whose selector matches, later blocks winning. */
export function readBlock(css: string, selector: string): Tokens {
  const tokens: Tokens = new Map();
  const withoutComments = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  for (const block of withoutComments.matchAll(new RegExp(`(?:^|\\}|;)\\s*${escaped}\\s*\\{([^{}]*)\\}`, "g"))) {
    for (const declaration of (block[1] ?? "").matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) {
      if (declaration[1] && declaration[2]) tokens.set(declaration[1], declaration[2].trim());
    }
  }
  return tokens;
}

/** The value of a color token in one scheme: the matching half of `light-dark(a, b)`, or the plain value. */
export function colorIn(tokens: Tokens, name: string, scheme: Scheme): string {
  const value = tokens.get(name);
  if (value === undefined) throw new Error(`token ${name} is missing`);
  const pair = /^light-dark\(\s*([^,()]+(?:\([^)]*\))?)\s*,\s*([^,()]+(?:\([^)]*\))?)\s*\)$/.exec(value);
  if (!pair?.[1] || !pair[2]) return value;
  return (scheme === "light" ? pair[1] : pair[2]).trim();
}
