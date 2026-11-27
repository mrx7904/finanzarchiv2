import { useCallback, useEffect, useState } from "react";
import { type RecurringRow_Serialize as RecurringRow, commands } from "../bindings";
import { Button, EmptyState, List, ListRow, LoadingRows, cn } from "../ui";
import { amountText, formatIsoDay } from "./describe";
import { recurringState, recurringSums } from "./recurrence";

// "Konten › Wiederkehrend" (#52): the sum of the running payments in the month and in the year, every payment with its
// rhythm, next day, amount and state, and beside the list the chosen one with its last twelve payments, its category,
// "Buchungen zeigen" and "Ist keine wiederkehrende Zahlung" (which stays, #34).
export function RecurringPage({ onBookings }: { onBookings: (text: string) => void }) {
  const [rows, setRows] = useState<RecurringRow[] | null>(null);
  const [doubled, setDoubled] = useState<Set<number>>(new Set());
  const [chosen, setChosen] = useState<string | null>(null);
  const reload = useCallback(async () => {
    const [listed, hints] = await Promise.all([commands.listRecurring(), commands.listHints()]);
    if (listed.status === "ok") setRows(listed.data);
    if (hints.status === "ok") setDoubled(new Set(hints.data.filter((hint) => hint.kind === "twice").flatMap((hint) => hint.bookings)));
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);

  const key = (row: RecurringRow) => `${row.party}-${row.incoming}-${row.rhythm}-${row.since}`;
  const selected = rows?.find((row) => key(row) === chosen) ?? null;
  if (rows !== null && rows.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <EmptyState icon="card" title="Keine wiederkehrende Zahlung" hint="Sie zeigt sich nach drei Zahlungen im gleichen Abstand." />
      </div>
    );
  }
  const sums = recurringSums(rows ?? []);
  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(300px,380px)] gap-4 px-6 pt-4 pb-6">
      <section aria-label="Wiederkehrend" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
        <div className="flex items-baseline gap-4 border-b border-line p-4">
          <span className="text-[26px] leading-tight font-semibold tabular-nums">{amountText(sums.month, false)}</span>
          <span className="text-small text-fg-muted tabular-nums">
            im Monat · {amountText(sums.year, false)} im Jahr · {sums.running} laufend
          </span>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto">
          {rows === null ? (
            <LoadingRows loading />
          ) : (
            <List label="Wiederkehrende Zahlungen">
              {rows.map((row) => (
                <ListRow
                  key={key(row)}
                  title={row.name}
                  detail={`${row.rhythm}${row.next ? ` · nächste ${formatIsoDay(row.next)}` : ""}`}
                  figure={`${row.incoming ? "+" : ""}${row.text}`}
                  state={recurringState(row, doubled)}
                  selected={key(row) === chosen}
                  onSelect={() => setChosen(key(row))}
                />
              ))}
            </List>
          )}
        </div>
      </section>
      <section aria-label="Zahlung" className="flex min-h-0 flex-col gap-4 overflow-y-auto rounded-panel bg-surface p-4 shadow-panel">
        {selected ? (
          <>
            <div className="flex flex-col gap-0.5">
              <span className="font-medium">{selected.name}</span>
              <span className="text-small text-fg-muted">
                {selected.rhythm}
                {selected.category ? ` · ${selected.category}` : ""}
              </span>
            </div>
            <History points={selected.history} />
            <div className="flex flex-col">
              <Line name="Nächste">{selected.next ? formatIsoDay(selected.next) : "beendet"}</Line>
              <Line name="Im Jahr">{selected.perYearText.replace("-", "")}</Line>
              {selected.change && <Line name="Betrag">{selected.change}</Line>}
              <Line name="Seit">{formatIsoDay(selected.since)}</Line>
            </div>
            <div className="mt-auto flex flex-wrap justify-end gap-2">
              <Button
                kind="plain"
                onClick={() =>
                  void commands.notRecurring(selected.party, selected.incoming, selected.rhythm, selected.since).then(() => {
                    setChosen(null);
                    void reload();
                  })
                }
              >
                Ist keine wiederkehrende Zahlung
              </Button>
              <Button onClick={() => onBookings(selected.name)}>Buchungen zeigen</Button>
            </div>
          </>
        ) : (
          <p className="text-fg-muted">Keine Zahlung gewählt</p>
        )}
      </section>
    </div>
  );
}

function Line({ name, children }: { name: string; children: React.ReactNode }) {
  return (
    <div className="flex min-h-9 items-center gap-3 border-b border-line py-1.5 last:border-b-0">
      <span className="w-24 shrink-0 text-small text-fg-muted">{name}</span>
      <span className="min-w-0 flex-1 truncate tabular-nums">{children}</span>
    </div>
  );
}

/** The last twelve payments as gray bars, the last one in the accent (rule 12), one scale for all. */
function History({ points }: { points: { day: string; cents: number; text: string }[] }) {
  const top = Math.max(1, ...points.map((point) => Math.abs(point.cents)));
  return (
    <div aria-label="Die letzten Zahlungen" className="flex h-24 items-end gap-1">
      {points.map((point, at) => (
        <span
          key={point.day}
          title={`${formatIsoDay(point.day)} · ${point.text}`}
          className={cn("flex-1 rounded-t-[3px]", at === points.length - 1 ? "bg-accent" : "bg-line-strong")}
          style={{ height: `${Math.max(4, (Math.abs(point.cents) / top) * 100)}%` }}
        />
      ))}
    </div>
  );
}
