import { useCallback, useEffect, useState } from "react";
import { type InvoiceCandidate, type OrderRow_Serialize as OrderRow, commands } from "../bindings";
import { formatIsoDay } from "../money/describe";
import { Button, Dialog, EmptyState, List, ListRow, LoadingRows, cn } from "../ui";
import { orderCountsLine, orderState } from "./describe";

// "Bestellungen" (#32), a page below Dokumente since #46: every order the mails of Amazon told, "fehlt" or
// "vollständig", beside it its invoice. "Rechnung zuordnen …" lists the invoices no order holds, the likeliest first;
// "Braucht keine Rechnung" ends the "fehlt" of an order that gets none, and "Rückgängig" brings it back (decided by
// Fable 5.2 for the owner on 2026-11-27). The app never signs in at Amazon.
export function OrdersPage({ onOpenDocument }: { onOpenDocument: (document: number) => void }) {
  const [orders, setOrders] = useState<OrderRow[] | null>(null);
  const [chosen, setChosen] = useState<number | null>(null);
  const [assigning, setAssigning] = useState(false);

  const reload = useCallback(async () => {
    const result = await commands.listOrders();
    if (result.status === "ok") setOrders(result.data);
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);

  if (orders !== null && orders.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <EmptyState icon="receipt" title="Noch keine Bestellung" hint="Bestellungen kommen aus den Mails von Amazon." />
      </div>
    );
  }
  const selected = orders?.find((order) => order.id === chosen) ?? null;
  const needs = async (order: number, value: boolean) => {
    await commands.setOrderNeedsInvoice(order, value);
    await reload();
  };
  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(320px,420px)_minmax(0,1fr)] gap-4 px-6 pt-4 pb-6">
      <section aria-label="Bestellungen" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
        <span className="border-b border-line p-3 text-small text-fg-muted tabular-nums">{orders ? orderCountsLine(orders) : ""}</span>
        <div className="min-h-0 flex-1 overflow-y-auto">
          {orders === null ? (
            <LoadingRows loading />
          ) : (
            <List label="Bestellungen">
              {orders.map((order) => (
                <ListRow
                  key={order.id}
                  title={`${order.shop} ${order.number}`}
                  detail={order.invoiceName ?? (order.noInvoice ? "Braucht keine Rechnung" : [order.day ? formatIsoDay(order.day) : null, order.total].filter(Boolean).join(" · "))}
                  state={orderState(order)}
                  selected={order.id === chosen}
                  onSelect={() => setChosen(order.id)}
                />
              ))}
            </List>
          )}
        </div>
      </section>
      <section aria-label="Bestellung" className="flex min-h-0 flex-col gap-3 self-start rounded-panel bg-surface p-4 shadow-panel">
        {selected ? (
          <>
            <span className="font-medium tabular-nums">
              {selected.shop} {selected.number}
            </span>
            <span className="text-small text-fg-muted tabular-nums">{[selected.day ? formatIsoDay(selected.day) : null, selected.total].filter(Boolean).join(" · ")}</span>
            <div className="flex items-center gap-2.5 rounded-row border border-line px-3 py-2">
              <span className={cn("size-2 shrink-0 rounded-full", selected.invoice !== null ? "bg-ok" : selected.noInvoice ? "bg-fg-muted" : "bg-danger")} />
              {selected.invoice !== null ? (
                <button type="button" className="min-w-0 flex-1 truncate text-left hover:underline" onClick={() => onOpenDocument(selected.invoice ?? 0)}>
                  {selected.invoiceName ?? "Rechnung"}
                </button>
              ) : selected.noInvoice ? (
                <>
                  <span className="flex-1 text-fg-muted">Braucht keine Rechnung</span>
                  <Button size="small" kind="plain" onClick={() => void needs(selected.id, true)}>
                    Rückgängig
                  </Button>
                </>
              ) : (
                <span className="flex-1">Rechnung fehlt</span>
              )}
            </div>
            {selected.invoice === null && (
              <div className="flex justify-end gap-2">
                {!selected.noInvoice && (
                  <Button kind="plain" onClick={() => void needs(selected.id, false)}>
                    Braucht keine Rechnung
                  </Button>
                )}
                <Button onClick={() => setAssigning(true)}>Rechnung zuordnen …</Button>
              </div>
            )}
            <AssignDialog
              open={assigning}
              order={selected.id}
              onClose={() => setAssigning(false)}
              onAssigned={() => {
                setAssigning(false);
                void reload();
              }}
            />
          </>
        ) : (
          <p className="text-fg-muted">Keine Bestellung gewählt</p>
        )}
      </section>
    </div>
  );
}

/** "Rechnung zuordnen …": the invoices no order holds, the likeliest first. Picking one is the dialog's answer. */
function AssignDialog({ open, order, onClose, onAssigned }: { open: boolean; order: number; onClose: () => void; onAssigned: () => void }) {
  const [candidates, setCandidates] = useState<InvoiceCandidate[] | null>(null);
  const [picked, setPicked] = useState<number | null>(null);
  useEffect(() => {
    if (!open) return;
    setPicked(null);
    void commands.orderInvoiceCandidates(order).then((result) => {
      if (result.status === "ok") setCandidates(result.data);
    });
  }, [open, order]);
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Welche Rechnung gehört zur Bestellung?"
      actions={
        <>
          <Button onClick={onClose}>Abbrechen</Button>
          <Button
            kind="primary"
            disabled={picked === null}
            onClick={() => {
              if (picked !== null) void commands.assignOrderInvoice(order, picked).then(onAssigned);
            }}
          >
            Zuordnen
          </Button>
        </>
      }
    >
      {candidates && candidates.length === 0 ? (
        <p className="text-fg-muted">Keine Rechnung ohne Bestellung</p>
      ) : (
        <List label="Rechnungen">
          {(candidates ?? []).map((candidate) => (
            <ListRow
              key={candidate.document}
              title={candidate.name}
              detail={candidate.day ? formatIsoDay(candidate.day) : undefined}
              figure={candidate.gross ?? ""}
              selected={picked === candidate.document}
              onSelect={() => setPicked(candidate.document)}
            />
          ))}
        </List>
      )}
    </Dialog>
  );
}
