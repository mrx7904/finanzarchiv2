// Turning pages and zooming a document (#54): pure, for its tests.

/** "fit" is the whole page in its column; a number is percent of the column's width. */
export type Zoom = "fit" | number;

export const ZOOMS: Zoom[] = ["fit", 50, 75, 100, 125, 150, 200, 300];

/** The next zoom smaller or larger, stopping at the ends. */
export function zoomStep(zoom: Zoom, delta: 1 | -1): Zoom {
  const at = Math.max(0, ZOOMS.indexOf(zoom));
  return ZOOMS[Math.min(ZOOMS.length - 1, Math.max(0, at + delta))] ?? "fit";
}

/** The page after a step, never before the first or after the last. */
export function pageStep(page: number, count: number, delta: number): number {
  return Math.min(Math.max(1, count), Math.max(1, page + delta));
}

export function zoomLabel(zoom: Zoom): string {
  return zoom === "fit" ? "Ganze Seite" : `${zoom} %`;
}

/** The width in pixels to ask the core for, so the page stays sharp on the screen it is shown on. */
export function requestWidth(column: number, zoom: Zoom, pixelRatio: number): number {
  const shown = zoom === "fit" ? column : (column * zoom) / 100;
  return Math.max(160, Math.round(shown * Math.max(1, pixelRatio)));
}
