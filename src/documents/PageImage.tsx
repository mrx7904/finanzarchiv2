import { useEffect, useRef, useState } from "react";
import { commands } from "../bindings";
import { cn } from "../ui";
import { loadPage } from "./pageCache";

/**
 * A page of a document as the core renders it. It asks only once it comes into view, so a list
 * of two hundred documents renders the pages one sees, and the core keeps each in its cache.
 *
 * While another width or page of the same document loads, the image shown before stays (#55): an empty box would
 * lose its height, the scroll bar of the column would come or go, and the new width would start the next request.
 */
/** `fit`: the whole page in its box, by width or by height, whichever is smaller (the page column of #46). */
export function PageImage({ document, page = 1, width, className, label, fit = false }: { document: number; page?: number; width: number; className?: string; label: string; fit?: boolean }) {
  const [shown, setShown] = useState<{ document: number; source: string } | null>(null);
  const [failed, setFailed] = useState(false);
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setFailed(false);
    const element = box.current;
    if (!element) return;
    let cancelled = false;
    const load = () =>
      loadPage(document, page, width, commands.pageImage).then((source) => {
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
    observer.observe(element);
    return () => {
      cancelled = true;
      observer.disconnect();
    };
  }, [document, page, width]);

  const source = shown?.document === document && !failed ? shown.source : null;
  return (
    <div ref={box} className={cn("overflow-hidden", fit ? "flex h-full w-full items-center justify-center" : "bg-canvas", failed && !fit && "flex aspect-[1/1.414] items-center justify-center", className)}>
      {source ? <img src={source} alt={label} className={fit ? "block max-h-full max-w-full rounded-control object-contain shadow-panel" : "block h-auto w-full"} /> : null}
      {failed ? <span className="text-small text-fg-muted">Keine Vorschau</span> : null}
    </div>
  );
}
