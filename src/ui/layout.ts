// What the owner fits in a view (#53): column widths, the rail, folded columns and the size. It belongs to the
// computer, not to the archive, so it is kept in the webview's storage under one key per thing, and read with its
// default when nothing or something broken is stored.

export type Size = "klein" | "mittel" | "gross";

/** The zoom of each size: 90, 100 and 115 % (rule 27 of knowledge/decision-views.md). */
export const SIZES: Record<Size, number> = { klein: 0.9, mittel: 1, gross: 1.15 };

const PREFIX = "finanzarchiv.layout.";

type Store = Pick<Storage, "getItem" | "setItem" | "removeItem">;

function storage(): Store | null {
  return typeof window === "undefined" ? null : window.localStorage;
}

export function readNumber(key: string, fallback: number, store: Store | null = storage()): number {
  const value = Number(store?.getItem(PREFIX + key));
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

export function readFlag(key: string, fallback: boolean, store: Store | null = storage()): boolean {
  const value = store?.getItem(PREFIX + key);
  return value === "1" ? true : value === "0" ? false : fallback;
}

export function readSize(store: Store | null = storage()): Size {
  const value = store?.getItem(PREFIX + "size");
  return value === "klein" || value === "gross" ? value : "mittel";
}

export function write(key: string, value: string | number | boolean | null, store: Store | null = storage()): void {
  if (value === null) store?.removeItem(PREFIX + key);
  else store?.setItem(PREFIX + key, typeof value === "boolean" ? (value ? "1" : "0") : String(value));
}

/** A width dragged or keyed, held between the column's least and greatest width. */
export function clampWidth(width: number, least: number, most: number): number {
  return Math.round(Math.min(Math.max(width, least), Math.max(least, most)));
}

/** The size one step smaller or larger, or back to Mittel. */
export function sizeStep(size: Size, key: "+" | "-" | "0"): Size {
  const order: Size[] = ["klein", "mittel", "gross"];
  if (key === "0") return "mittel";
  const at = order.indexOf(size) + (key === "+" ? 1 : -1);
  return order[Math.min(order.length - 1, Math.max(0, at))] ?? "mittel";
}
