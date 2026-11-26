import { useEffect, useState } from "react";
import { type OrderRow, commands } from "../bindings";
import { formatIsoDay } from "../money/describe";
import { Button, Dialog } from "../ui";
import { ordersLine } from "./describe";

// "Bestellungen" of #32: the orders the mails of Amazon told, newest first, each with its invoice
// or the hint to download it. The app never signs in at Amazon (decided 2026-11-26).
export function Orders({ open, onClose, onOpenDocument }: { open: boolean; onClose: () => void; onOpenDocument: (document: number) => void }) {
  const [orders, setOrders] = useState<OrderRow[]>([]);

  useEffect(() => {
    if (!open) return;
    void commands.listOrders().then((result) => {
      if (result.status === "ok") setOrders(result.data);
    });
  }, [open]);

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Bestellungen"
      description={ordersLine(orders)}
      actions={
        <Button kind="primary" onClick={onClose}>
          Fertig
        </Button>
      }
    >
      {orders.length === 0 ? (
        <p className="text-small text-fg-muted">Noch keine Bestellung aus den Mails von Amazon.</p>
      ) : (
        <ul className="flex max-h-[28rem] flex-col divide-y divide-line overflow-auto">
          {orders.map((order) => (
            <li key={order.id} className="flex items-center justify-between gap-3 py-2">
              <span className="min-w-0">
                <span className="block text-body tabular-nums">
                  {order.shop} {order.number}
                </span>
                <span className="block text-small text-fg-muted tabular-nums">
                  {[order.day ? formatIsoDay(order.day) : null, order.total].filter(Boolean).join(" · ")}
                </span>
              </span>
              {typeof order.invoice === "number" ? (
                <Button size="small" kind="plain" onClick={() => onOpenDocument(order.invoice ?? 0)}>
                  {order.invoiceName ?? "Rechnung"}
                </Button>
              ) : (
                <span className="shrink-0 text-small text-fg-muted">Rechnung fehlt</span>
              )}
            </li>
          ))}
        </ul>
      )}
    </Dialog>
  );
}
