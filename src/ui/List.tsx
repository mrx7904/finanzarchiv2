// A list of rows to pick one from, such as the documents beside their preview: a mark in front, a title with one line
// below it, the figure and the state at the end. The arrow keys move between rows, Home and End jump, and the
// chosen row stays tinted. A state is a capsule of one width for every state, neutral with a colored dot, so the
// column of states stands calm and still reads at a glance (asked for by the owner on 2026-11-25: one size, accented,
// not filled with color).
// Adapted from Beautiful UI "Task Rows" (github.com/slev12397/beautiful-ui, components/primitives/TaskRows.tsx, MIT,
// Copyright (c) 2026 Shane Levine): the row of badge, label, amount and capsule. Its timed demo sequence, the
// expanding details and the staggered entry are left out, a list of the archive does not perform.
import { useRef, type KeyboardEvent, type ReactNode } from "react";
import { cn } from "./motion";

export type Tone = "ok" | "warn" | "danger" | "neutral";

const dots: Record<Tone, string> = {
  ok: "bg-ok",
  warn: "bg-warn",
  danger: "bg-danger",
  neutral: "bg-fg-muted",
};

/** One width for every state, so a column of them lines up. Room for about ten letters. */
export function Capsule({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <span className="inline-flex h-5.5 w-24 shrink-0 items-center gap-1.5 rounded-full bg-surface px-2 text-small font-medium text-fg shadow-[inset_0_0_0_1px_var(--line-strong)]">
      <span aria-hidden className={cn("size-2 shrink-0 rounded-full", dots[tone])} />
      <span className="truncate">{children}</span>
    </span>
  );
}

/** The rows of one list. `label` names it for a screen reader. */
export function List({ label, children, className }: { label: string; children: ReactNode; className?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const onKeyDown = (event: KeyboardEvent) => {
    const rows = [...(ref.current?.querySelectorAll<HTMLElement>("[role='option']") ?? [])];
    const at = rows.indexOf(document.activeElement as HTMLElement);
    const next = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: rows.length - 1 }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    const row = rows[Math.max(0, Math.min(rows.length - 1, next))];
    row?.focus();
    row?.click();
  };
  return (
    <div ref={ref} role="listbox" aria-label={label} onKeyDown={onKeyDown} className={cn("flex flex-col gap-px p-1", className)}>
      {children}
    </div>
  );
}

type ListRowProps = {
  title: string;
  detail?: string;
  /** Before the title: an icon or an initial. */
  leading?: ReactNode;
  /** The figure before the state, set in tabular digits. An empty string keeps the column for a row without one. */
  figure?: string;
  state?: { tone: Tone; label: string };
  selected?: boolean;
  onSelect: () => void;
};

export function ListRow({ title, detail, leading, figure, state, selected = false, onSelect }: ListRowProps) {
  return (
    <div
      role="option"
      aria-selected={selected}
      tabIndex={selected ? 0 : -1}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "relative flex min-h-(--row-height) cursor-default items-center gap-3 rounded-row px-2.5 py-1.5 outline-offset-[-2px] transition-quick select-none",
        selected ? "bg-selected" : "hover:bg-hover",
      )}
    >
      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
      {leading && <span className="flex size-8 shrink-0 items-center justify-center rounded-control bg-canvas text-fg-muted">{leading}</span>}
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium text-fg">{title}</span>
        {detail && <span className="truncate text-small text-fg-muted">{detail}</span>}
      </span>
      {figure !== undefined && <span className="shrink-0 font-medium text-fg tabular-nums">{figure}</span>}
      {state && <Capsule tone={state.tone}>{state.label}</Capsule>}
    </div>
  );
}

type DenseRowProps = {
  /** The first column, such as the day. */
  lead?: string;
  title: string;
  /** Text after the title on the same line, dropped first when the row is narrow. */
  detail?: string;
  figure?: string;
  selected?: boolean;
  onSelect: () => void;
  /** Buttons at the right that show on hover or focus. The row keeps its height either way (rule 8). */
  actions?: ReactNode;
};

/** A row of 40 px for long lists such as Umsätze (rule 7), with the arrow keys of `List`. */
export function DenseRow({ lead, title, detail, figure, selected = false, onSelect, actions }: DenseRowProps) {
  return (
    <div
      role="option"
      aria-selected={selected}
      tabIndex={selected ? 0 : -1}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "group relative flex h-(--dense-height) cursor-default items-center gap-3 rounded-row px-2.5 outline-offset-[-2px] transition-quick select-none",
        selected ? "bg-selected" : "hover:bg-hover",
      )}
    >
      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
      {lead && <span className="w-12 shrink-0 text-small text-fg-muted tabular-nums">{lead}</span>}
      <span className="min-w-0 flex-1 truncate">
        <span className="font-medium text-fg">{title}</span>
        {detail && <span className="text-fg-muted"> · {detail}</span>}
      </span>
      {actions && <RowActions>{actions}</RowActions>}
      {figure !== undefined && <span className="w-28 shrink-0 text-right font-medium text-fg tabular-nums">{figure}</span>}
    </div>
  );
}

/** The actions of a row: hidden until the row is hovered or holds the focus, never changing its height. */
export function RowActions({ children }: { children: ReactNode }) {
  return (
    <span className="flex shrink-0 items-center gap-1 opacity-0 transition-quick group-focus-within:opacity-100 group-hover:opacity-100 focus-within:opacity-100">
      {children}
    </span>
  );
}

/** The head of a group of rows, such as a month with its sums. It stays at the top while its rows scroll. */
export function GroupHead({ title, figures }: { title: string; figures?: ReactNode }) {
  return (
    <div className="sticky top-0 z-10 flex h-8 items-center gap-3 bg-surface/95 px-2.5 text-small font-medium text-fg-muted backdrop-blur-sm">
      <span className="flex-1 truncate">{title}</span>
      {figures && <span className="flex shrink-0 items-center gap-3 tabular-nums">{figures}</span>}
    </div>
  );
}

/** A filter a jump has set, as a mark with "×" in the head of a list (rule 9). */
export function FilterMark({ label, onClear }: { label: string; onClear: () => void }) {
  return (
    <span className="inline-flex h-6 max-w-64 items-center gap-1 rounded-full bg-selected pr-0.5 pl-2.5 text-small font-medium text-fg">
      <span className="truncate">{label}</span>
      <button
        type="button"
        aria-label={`Filter „${label}“ entfernen`}
        onClick={onClear}
        className="flex size-5 shrink-0 items-center justify-center rounded-full text-fg-muted transition-quick hover:bg-hover hover:text-fg"
      >
        ×
      </button>
    </span>
  );
}
