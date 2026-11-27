import { type ReactNode, useEffect, useRef, useState } from "react";
import { commands } from "../bindings";
import { cn } from "../ui";
import { loadPage } from "./pageCache";
import { type Rect, containedRect, requestWidth } from "./pages";

/**
 * A page of a document as the core renders it. It asks only once it comes into view, so a list
 * of two hundred documents renders the pages one sees, and the core keeps each in its cache.
 *
 * While another width or page of the same document loads, the image shown before stays (#55): an empty box would
 * lose its height, the scroll bar of the column would come or go, and the new width would start the next request.
 *
 * `overlay` is drawn on the image itself, not on its box, so a mark in fractions of the page lies on its line at every
 * zoom (#57). Without `width` a fitted page is asked at the width of its box.
 */
/** `fit`: the whole page in its box, by width or by height, whichever is smaller (the page column of #46). */
export function PageImage({ document, page = 1, width, className, label, fit = false, overlay }: { document: number; page?: number; width?: number; className?: string; label: string; fit?: boolean; overlay?: ReactNode }) {
  const [shown, setShown] = useState<{ document: number; source: string } | null>(null);
  const [failed, setFailed] = useState(false);
  const [box, setBox] = useState({ width: 0, height: 0 });
  const [natural, setNatural] = useState({ width: 0, height: 0 });
  const element = useRef<HTMLDivElement>(null);

  // A fitted page knows the size of its box, to place the overlay and, without `width`, to ask at that width.
  useEffect(() => {
    const at = element.current;
    if (!at || !fit) return;
    const observer = new ResizeObserver(([entry]) => {
      const rect = entry?.contentRect;
      if (rect) setBox((was) => (was.width === Math.round(rect.width) && was.height === Math.round(rect.height) ? was : { width: Math.round(rect.width), height: Math.round(rect.height) }));
    });
    observer.observe(at);
    return () => observer.disconnect();
  }, [fit]);

  const ratio = typeof window === "undefined" ? 1 : window.devicePixelRatio;
  const asked = width ?? (box.width > 0 ? requestWidth(box.width, "fit", ratio) : 0);

  useEffect(() => {
    setFailed(false);
    const at = element.current;
    if (!at || asked <= 0) return;
    let cancelled = false;
    const load = () =>
      loadPage(document, page, asked, commands.pageImage).then((source) => {
        if (cancelled) return;
        if (source === null) setFailed(true);
        else setShown({ document, source });
      });
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        observer.disconnect();
        void load();
      }
    });
    observer.observe(at);
    return () => {
      cancelled = true;
      observer.disconnect();
    };
  }, [document, page, asked]);

  const source = shown?.document === document && !failed ? shown.source : null;
  const layer: Rect | null = !source || !overlay ? null : fit ? containedRect(box, natural) : null;
  return (
    <div ref={element} className={cn("relative overflow-hidden", fit ? "flex h-full w-full items-center justify-center" : "bg-canvas", failed && !fit && "flex aspect-[1/1.414] items-center justify-center", className)}>
      {source ? (
        <img
          src={source}
          alt={label}
          onLoad={(event) => setNatural({ width: event.currentTarget.naturalWidth, height: event.currentTarget.naturalHeight })}
          className={fit ? "block max-h-full max-w-full rounded-control object-contain shadow-panel" : "block h-auto w-full"}
        />
      ) : null}
      {source && overlay ? (
        <div data-page-layer className="pointer-events-none absolute" style={layer ? { left: layer.left, top: layer.top, width: layer.width, height: layer.height } : { inset: 0 }}>
          {overlay}
        </div>
      ) : null}
      {failed ? <span className="text-small text-fg-muted">Keine Vorschau</span> : null}
    </div>
  );
}
