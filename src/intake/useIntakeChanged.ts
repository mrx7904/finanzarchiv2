import { useEffect, useRef } from "react";
import { events } from "../bindings";

/**
 * Calls `reload` when the queue changed, at most every 150 ms: two hundred files report two
 * hundred changes, and one reload a moment later shows them all.
 */
export function useIntakeChanged(reload: () => void) {
  const latest = useRef(reload);
  latest.current = reload;
  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | null = null;
    const stop = events.intakeChanged.listen(() => {
      if (timer !== null) return;
      timer = setTimeout(() => {
        timer = null;
        latest.current();
      }, 150);
    });
    return () => {
      if (timer !== null) clearTimeout(timer);
      void stop.then((unlisten) => unlisten());
    };
  }, []);
}
