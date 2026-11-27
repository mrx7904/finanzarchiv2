// Turning pages and zooming a document (#54): pure, for its tests.
import { PAGE_WIDTHS } from "../bindings";

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

/**
 * The width in pixels to ask the core for, so the page stays sharp on the screen it is shown on. It is one of the
 * widths the core renders (`previews::WIDTHS`), so a column a few pixels narrower or wider asks nothing new (#55).
 */
export function requestWidth(column: number, zoom: Zoom, pixelRatio: number): number {
  const shown = zoom === "fit" ? column : (column * zoom) / 100;
  const wanted = Math.round(shown * Math.max(1, pixelRatio));
  return PAGE_WIDTHS.find((width) => width >= wanted) ?? PAGE_WIDTHS[PAGE_WIDTHS.length - 1] ?? wanted;
}

export type Rect = { left: number; top: number; width: number; height: number };

/**
 * Where an image of `image` pixels stands when it is fitted whole into a box of `box` pixels and centred, as
 * `object-fit: contain` places it. A mark is placed on this rectangle, not on the box, which is wider than a portrait
 * page and higher than a landscape one (#57).
 */
export function containedRect(box: { width: number; height: number }, image: { width: number; height: number }): Rect {
  if (box.width <= 0 || box.height <= 0 || image.width <= 0 || image.height <= 0) return { left: 0, top: 0, width: 0, height: 0 };
  const scale = Math.min(1, box.width / image.width, box.height / image.height);
  const width = image.width * scale;
  const height = image.height * scale;
  return { left: (box.width - width) / 2, top: (box.height - height) / 2, width, height };
}

/** A mark of fractions (left, top, width, height of the page) on the rectangle of the image, in pixels. */
export function markRect(image: Rect, mark: readonly (number | null)[]): Rect {
  const at = (index: number) => mark[index] ?? 0;
  return { left: image.left + at(0) * image.width, top: image.top + at(1) * image.height, width: at(2) * image.width, height: at(3) * image.height };
}
