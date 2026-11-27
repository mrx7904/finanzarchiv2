// The line between two columns of a list with detail (#53, rule 23 of knowledge/decision-views.md): dragged by the
// mouse, moved by the arrow keys with the focus on it, a double click gives the width of the design back. No column
// gets narrower than its least width, and the width is kept on the computer per view.
// After the idea of the column resizers listed on 21st.dev (idea only): a thin handle that shows on hover and focus.
import { type CSSProperties, type KeyboardEvent, type PointerEvent, type ReactNode, useState } from "react";
import { clampWidth, readNumber, write } from "./layout";
import { cn } from "./motion";

type ColumnOptions = {
  /** Where the width is kept, such as "documents.list". */
  key: string;
  initial: number;
  least: number;
  most: number;
  /** "left": the column left of the line is sized; "right": the one right of it. */
  side?: "left" | "right";
  label: string;
};

/** The width of a resizable column and the line that resizes it, to be placed inside the sized column. */
export function useColumn({ key, initial, least, most, side = "left", label }: ColumnOptions): [number, ReactNode] {
  const [width, setWidthHere] = useState(() => clampWidth(readNumber(key, initial), least, most));
  const setWidth = (next: number) => {
    const held = clampWidth(next, least, most);
    setWidthHere(held);
    write(key, held === initial ? null : held);
  };
  const sign = side === "left" ? 1 : -1;
  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    const start = event.clientX;
    const from = width;
    const target = event.currentTarget;
    target.setPointerCapture(event.pointerId);
    const move = (moved: globalThis.PointerEvent) => setWidth(from + sign * (moved.clientX - start));
    const up = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", up);
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", up);
  };
  const onKeyDown = (event: KeyboardEvent) => {
    const step = { ArrowLeft: -16, ArrowRight: 16 }[event.key];
    if (step === undefined) return;
    event.preventDefault();
    setWidth(width + sign * step);
  };
  const divider = (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={label}
      aria-valuenow={width}
      aria-valuemin={least}
      aria-valuemax={most}
      tabIndex={0}
      title="Ziehen, Doppelklick stellt zurück"
      onPointerDown={onPointerDown}
      onKeyDown={onKeyDown}
      onDoubleClick={() => setWidth(initial)}
      className={cn(
        "group/divider absolute top-3 bottom-3 z-10 flex w-3 cursor-col-resize justify-center outline-none",
        side === "left" ? "-right-[14px]" : "-left-[14px]",
      )}
    >
      <span className="w-[3px] rounded-full bg-transparent transition-quick group-hover/divider:bg-line-strong group-focus-visible/divider:bg-accent" />
    </div>
  );
  return [width, divider];
}

/** The style that hands a column width to a grid template as a CSS variable. */
export function columnStyle(name: string, width: number): CSSProperties {
  return { [name]: `${width}px` } as CSSProperties;
}
