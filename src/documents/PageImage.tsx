import { useEffect, useRef, useState } from "react";
import { commands } from "../bindings";
import { cn } from "../ui";

/**
 * A page of a document as the core renders it. It asks only once it comes into view, so a list
 * of two hundred documents renders the pages one sees, and the core keeps each in its cache.
 */
export function PageImage({ document, page = 1, width, className, label }: { document: number; page?: number; width: number; className?: string; label: string }) {
  const [source, setSource] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setSource(null);
    setFailed(false);
    const element = box.current;
    if (!element) return;
    let cancelled = false;
    const load = () =>
      commands.pageImage(document, page, width).then((result) => {
        if (cancelled) return;
        if (result.status === "ok") setSource(result.data);
        else setFailed(true);
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

  return (
    <div ref={box} className={cn("overflow-hidden bg-canvas", className)}>
      {source ? <img src={source} alt={label} className="block h-auto w-full" /> : null}
      {failed ? <span className="sr-only">Keine Vorschau</span> : null}
    </div>
  );
}
