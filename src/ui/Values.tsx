// A row of a read value beside the page of a document (#45): its name, the value, and where it stands as a dot and
// a word: read from the page, checked by the archive, or given by the sender in its e-invoice. Drawn for this app
// after the rows of `List`, the dot in the colors of the states (rule 11 of knowledge/decision-views.md).
import type { ReactNode } from "react";
import { cn } from "./motion";

export type ValueSource = "read" | "checked" | "sender" | "owner";

const sources: Record<ValueSource, { word: string; dot: string }> = {
  read: { word: "gelesen", dot: "bg-fg-muted" },
  checked: { word: "stimmt", dot: "bg-ok" },
  sender: { word: "vom Absender", dot: "bg-ok" },
  owner: { word: "von dir", dot: "bg-fg-muted" },
};

type ValueRowProps = {
  name: string;
  value: ReactNode;
  source?: ValueSource;
  /** Amounts stand right-aligned in tabular digits (rule 13). */
  amount?: boolean;
};

export function ValueRow({ name, value, source, amount = false }: ValueRowProps) {
  const shown = source ? sources[source] : undefined;
  return (
    <div className="flex min-h-9 items-center gap-3 border-b border-line py-1.5 last:border-b-0">
      <span className="w-24 shrink-0 text-small text-fg-muted">{name}</span>
      <span className={cn("min-w-0 flex-1 truncate text-fg", amount && "text-right font-medium tabular-nums")}>{value}</span>
      {shown && (
        <span className="inline-flex w-24 shrink-0 items-center gap-1.5 text-small whitespace-nowrap text-fg-muted">
          <span aria-hidden className={cn("size-2 shrink-0 rounded-full", shown.dot)} />
          {shown.word}
        </span>
      )}
    </div>
  );
}
