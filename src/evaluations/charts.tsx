// The charts of the evaluations (#37, #38): plain HTML and SVG without a chart library. One y scale per chart (a
// comparison never gets two), thin marks with 4 px round ends at the baseline, a 2 px gap between bars, a missing month
// as "fehlt", text at its size at every width (the bars are HTML, the line keeps its stroke under scaling), and every
// mark tells its value on hover. Colors come from the caller: Graphit's grays with the accent for the chosen mark
// (rule 12 of knowledge/decision-views.md), or the validated palette of the dataviz skill.
import type { ReactNode } from "react";
import { cn } from "../ui";

export const MONTH_LETTERS = ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
const MONTH_NAMES = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];

/** Cents as the window shows them; money in with a "+" when asked. */
export function euros(cents: number, plus = false): string {
  const magnitude = Math.abs(cents);
  const text = `${Math.floor(magnitude / 100).toLocaleString("de-DE")},${String(magnitude % 100).padStart(2, "0")} €`;
  return cents < 0 ? `-${text}` : plus ? `+${text}` : text;
}

/** Whole euros for the labels of an axis. */
function short(cents: number): string {
  return `${Math.round(cents / 100).toLocaleString("de-DE")} €`;
}

export type Series = { name: string; values: (number | null)[]; color: string };

/** Bars by month, one or two series side by side on one scale. `null` is a month without data: "fehlt" when `missing`. */
export function MonthBars({ series, chosen, onChoose, missing = false, height = 180 }: { series: Series[]; chosen?: number | null; onChoose?: (month: number) => void; missing?: boolean; height?: number }) {
  const top = Math.max(1, ...series.flatMap((one) => one.values.map((value) => Math.abs(value ?? 0))));
  return (
    <div className="flex flex-col gap-2">
      {series.length > 1 && <Legend series={series} />}
      <div className="relative flex items-end gap-[6px]" style={{ height }}>
        <span className="pointer-events-none absolute top-0 right-0 text-small text-fg-muted tabular-nums">{short(top)}</span>
        {MONTH_LETTERS.map((_, month) => {
          const empty = series.every((one) => one.values[month] === null || one.values[month] === undefined);
          return (
            <button
              key={month}
              type="button"
              onClick={() => onChoose?.(month)}
              aria-label={`${MONTH_NAMES[month]}: ${series.map((one) => `${one.name} ${one.values[month] === null ? "fehlt" : euros(one.values[month] ?? 0)}`).join(", ")}`}
              title={`${MONTH_NAMES[month]}\n${series.map((one) => `${one.name}: ${one.values[month] === null ? "fehlt" : euros(one.values[month] ?? 0)}`).join("\n")}`}
              className={cn("group relative flex h-full flex-1 items-end justify-center gap-[2px] rounded-[4px] px-[2px]", chosen === month ? "bg-selected" : "hover:bg-hover")}
            >
              {empty && missing ? (
                <span className="absolute bottom-0 w-full text-center text-[11px] text-fg-muted">fehlt</span>
              ) : (
                series.map((one) => (
                  <span
                    key={one.name}
                    className="w-full max-w-5 rounded-t-[4px]"
                    style={{ height: `${(Math.abs(one.values[month] ?? 0) / top) * 100}%`, minHeight: one.values[month] ? 2 : 0, background: chosen === month && series.length === 1 ? "var(--accent)" : one.color }}
                  />
                ))
              )}
            </button>
          );
        })}
      </div>
      <div className="flex gap-[6px] border-t border-line pt-1">
        {MONTH_LETTERS.map((letter, month) => (
          <span key={month} className={cn("flex-1 text-center text-[11px] tabular-nums", chosen === month ? "font-semibold text-fg" : "text-fg-muted")}>
            {letter}
          </span>
        ))}
      </div>
    </div>
  );
}

export function Legend({ series }: { series: { name: string; color: string }[] }) {
  return (
    <div className="flex gap-4 text-small text-fg-muted">
      {series.map((one) => (
        <span key={one.name} className="inline-flex items-center gap-1.5">
          <span aria-hidden className="size-2.5 rounded-[3px]" style={{ background: one.color }} />
          {one.name}
        </span>
      ))}
    </div>
  );
}

/** Horizontal bars, the larger first, each with its name and amount as text beside the mark. */
export function RankBars({ parts, color, chosen, onChoose, limit = 8 }: { parts: { name: string; cents: number }[]; color: string; chosen?: string | null; onChoose?: (name: string) => void; limit?: number }) {
  const shown = parts.slice(0, limit);
  const top = Math.max(1, ...shown.map((part) => Math.abs(part.cents)));
  return (
    <div className="flex flex-col gap-1">
      {shown.map((part) => (
        <button
          key={part.name}
          type="button"
          onClick={() => onChoose?.(part.name)}
          title={`${part.name}: ${euros(part.cents)}`}
          className={cn("grid grid-cols-[minmax(0,9rem)_minmax(0,1fr)_6.5rem] items-center gap-3 rounded-row px-2 py-1 text-left", chosen === part.name ? "bg-selected" : "hover:bg-hover")}
        >
          <span className="truncate text-small">{part.name}</span>
          <span className="h-2.5 rounded-r-[4px]" style={{ width: `${Math.max(1, (Math.abs(part.cents) / top) * 100)}%`, background: chosen === part.name ? "var(--accent)" : color }} />
          <span className="text-right text-small font-medium tabular-nums">{euros(part.cents)}</span>
        </button>
      ))}
    </div>
  );
}

/** A line of balances on one scale, the last point marked; the stroke keeps its width at every size. */
export function BalanceLine({ values, labels, color, height = 140 }: { values: number[]; labels: string[]; color: string; height?: number }) {
  const low = Math.min(0, ...values);
  const high = Math.max(1, ...values);
  const x = (at: number) => (values.length > 1 ? (at / (values.length - 1)) * 100 : 50);
  const y = (value: number) => 100 - ((value - low) / (high - low)) * 100;
  const path = values.map((value, at) => `${at === 0 ? "M" : "L"}${x(at)},${y(value)}`).join(" ");
  const last = values.at(-1) ?? 0;
  return (
    <div className="flex flex-col gap-1">
      <div className="relative" style={{ height }}>
        <svg viewBox="0 0 100 100" preserveAspectRatio="none" className="absolute inset-0 size-full overflow-visible" aria-label={`Kontostand zuletzt ${euros(last)}`}>
          <line x1="0" x2="100" y1={y(0)} y2={y(0)} stroke="var(--line)" strokeWidth="1" vectorEffect="non-scaling-stroke" />
          <path d={path} fill="none" stroke={color} strokeWidth="2" strokeLinejoin="round" vectorEffect="non-scaling-stroke" />
        </svg>
        {values.map((value, at) => (
          <span key={labels[at]} title={`${labels[at]}: ${euros(value)}`} className="absolute size-3 -translate-x-1/2 -translate-y-1/2 rounded-full" style={{ left: `${x(at)}%`, top: `${y(value)}%` }} />
        ))}
        <span className="absolute size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full ring-2 ring-surface" style={{ left: `${x(values.length - 1)}%`, top: `${y(last)}%`, background: "var(--accent)" }} />
      </div>
      <div className="flex justify-between text-[11px] text-fg-muted tabular-nums">
        <span>
          {labels[0]} · {euros(values[0] ?? 0)}
        </span>
        <span>
          {labels.at(-1)} · {euros(last)}
        </span>
      </div>
    </div>
  );
}

/** A key figure: its name, the figure, one quiet line below (the year before). */
export function Figure({ label, value, detail, large = false }: { label: string; value: string; detail?: ReactNode; large?: boolean }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-small text-fg-muted">{label}</span>
      <span className={cn("font-semibold tracking-tight tabular-nums", large ? "text-[34px] leading-tight" : "text-[22px] leading-snug")}>{value}</span>
      {detail && <span className="text-small text-fg-muted tabular-nums">{detail}</span>}
    </div>
  );
}
