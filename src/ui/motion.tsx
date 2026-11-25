// The one gate to Motion: every piece imports `m`, `AnimatePresence` and the timings from here, never from
// "motion/react" itself, so reduced motion is decided in one place (checked by motion.test.ts).
// Motion and clsx are the picks of the skill pick-ui-library (github.com/emilkowalski/skills, MIT packages).
// Motion's own `reducedMotion="user"` stops transforms and layout only, a fade or a height would still run. So under
// reduced motion of the system the gate sets `MotionGlobalConfig.skipAnimations`, which makes every animation of
// Motion jump to its end, and follows the setting while the app runs.
import { LazyMotion, MotionConfig, MotionGlobalConfig, domMax, type Transition } from "motion/react";
import { useEffect, type ReactNode } from "react";

export { AnimatePresence, m } from "motion/react";
export { clsx as cn } from "clsx";

export const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

/** Something that moves to a place: no bounce, about a quarter of a second. */
export const spring: Transition = { type: "spring", bounce: 0, duration: 0.28 };
/** A fade or a small pop. */
export const quick: Transition = { duration: 0.16, ease: [0.23, 1, 0.32, 1] };

type ReducedMotionQuery = Pick<MediaQueryList, "matches" | "addEventListener" | "removeEventListener">;

export function applyReducedMotion(reduced: boolean): void {
  MotionGlobalConfig.skipAnimations = reduced;
}

/** Applies the system's setting now and on every change, and returns the function that stops listening. */
export function followReducedMotion(query: ReducedMotionQuery): () => void {
  const update = () => applyReducedMotion(query.matches);
  update();
  query.addEventListener("change", update);
  return () => query.removeEventListener("change", update);
}

// Set before the first render, since the children of MotionRoot start their animations before its effect runs.
if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
  applyReducedMotion(window.matchMedia(REDUCED_MOTION_QUERY).matches);
}

/** Wraps the app once. `strict`, so a `motion.div`, which would load every feature, is an error, only `m` exists. */
export function MotionRoot({ children }: { children: ReactNode }) {
  useEffect(() => followReducedMotion(window.matchMedia(REDUCED_MOTION_QUERY)), []);
  return (
    <LazyMotion features={domMax} strict>
      <MotionConfig reducedMotion="user">{children}</MotionConfig>
    </LazyMotion>
  );
}
