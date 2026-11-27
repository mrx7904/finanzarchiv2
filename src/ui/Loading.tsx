// Gray rows in the shape of a list while it loads (#45, rule 15 of knowledge/decision-views.md): what comes within
// 200 ms shows nothing, after that the rows stand, never a spinner in the middle of the window. The rows do not
// shimmer, so they look the same under reduced motion.
// After the idea of "Skeleton" by shadcn on 21st.dev (licence per component, idea only) and Magic UI's loading
// states: blocks in the shape of the content in the color of a hairline.
import { useEffect, useState } from "react";
import { cn } from "./motion";

/** The time a load may take before the gray rows show. */
export const LOADING_DELAY_MS = 200;

/** True once `active` has lasted `delay` ms, false again as soon as it ends. */
export function useDelayed(active: boolean, delay = LOADING_DELAY_MS): boolean {
  const [shown, setShown] = useState(false);
  useEffect(() => {
    if (!active) {
      setShown(false);
      return;
    }
    const timer = window.setTimeout(() => setShown(true), delay);
    return () => window.clearTimeout(timer);
  }, [active, delay]);
  return active && shown;
}

type LoadingRowsProps = {
  /** Whether the list loads. */
  loading: boolean;
  rows?: number;
  /** "dense" for the 40 px rows of Umsätze, "regular" for the rows of `List`. */
  density?: "dense" | "regular";
  label?: string;
};

/** Nothing for the first 200 ms of a load, then gray rows. */
export function LoadingRows({ loading, rows = 6, density = "regular", label = "Wird geladen" }: LoadingRowsProps) {
  const shown = useDelayed(loading);
  if (!shown) return null;
  return (
    <div role="status" aria-label={label} className="flex flex-col gap-px p-1">
      {Array.from({ length: rows }, (_, index) => (
        <div
          key={index}
          aria-hidden
          className={cn("flex items-center gap-3 rounded-row px-2.5", density === "dense" ? "h-(--dense-height)" : "h-(--row-height)")}
        >
          {density === "regular" && <span className="size-8 shrink-0 rounded-control bg-line" />}
          <span className="flex min-w-0 flex-1 flex-col gap-1.5">
            <span className="h-2.5 rounded-full bg-line" style={{ width: `${68 - ((index * 13) % 30)}%` }} />
            {density === "regular" && <span className="h-2 w-1/3 rounded-full bg-line" />}
          </span>
          <span className="h-2.5 w-16 shrink-0 rounded-full bg-line" />
        </div>
      ))}
    </div>
  );
}
