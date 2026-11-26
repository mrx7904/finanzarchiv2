import { useCallback, useEffect, useState } from "react";
import { type BookingPage, type BookingRow, type CategoryRow, commands } from "../bindings";
import { PageImage } from "../documents/PageImage";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { Button, SelectField, TextField, cn } from "../ui";
import { bookingFilter, bookingWho, formatIsoDay } from "./describe";

// "Umsätze" of #26: the bookings of one account or of all, newest first, 200 at a time, with their
// category, documents, moves and statement.
export function Bookings({ account }: { account: number | null }) {
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [category, setCategory] = useState("");
  const [text, setText] = useState("");
  const [page, setPage] = useState<BookingPage | null>(null);
  const [rows, setRows] = useState<BookingRow[]>([]);
  const [categories, setCategories] = useState<CategoryRow[]>([]);
  const [open, setOpen] = useState<number | null>(null);

  const load = useCallback(
    async (offset: number) => {
      const result = await commands.listBookings(bookingFilter(account, from, to, category, text, offset));
      if (result.status !== "ok") return;
      setPage(result.data);
      setRows((before) => (offset === 0 ? result.data.rows : [...before, ...result.data.rows]));
    },
    [account, from, to, category, text],
  );
  useEffect(() => {
    void load(0);
  }, [load]);
  useEffect(() => {
    void commands.listCategories().then((result) => {
      if (result.status === "ok") setCategories(result.data);
    });
  }, []);
  useIntakeChanged(() => void load(0));

  const options = [{ value: "", label: "Alle Kategorien" }, ...categories.map((row) => ({ value: String(row.id), label: row.name }))];
  const choose = async (booking: BookingRow, value: string, forParty: boolean) => {
    await commands.chooseCategory(booking.id, value === "" ? null : Number(value), forParty);
    void load(0);
  };

  return (
    <section className="flex w-full max-w-5xl flex-col gap-3">
      <div className="flex flex-wrap items-end gap-2">
        <TextField label="Von" type="date" value={from} onChange={setFrom} />
        <TextField label="Bis" type="date" value={to} onChange={setTo} />
        <SelectField label="Kategorie" value={category} onChange={setCategory} options={options} />
        <TextField label="Suche" icon="search" value={text} onChange={setText} clearable className="min-w-48 flex-1" />
      </div>
      {page ? (
        <p className="text-small text-fg-muted">
          {page.total} Buchungen, Einnahmen {page.income}, Ausgaben {page.spending}, Umbuchungen nicht mitgezählt
        </p>
      ) : null}
      <ul className="flex flex-col divide-y divide-line rounded-panel bg-surface shadow-panel" aria-label="Umsätze">
        {rows.map((booking) => (
          <li key={booking.id} className="flex flex-col gap-2 px-3 py-2">
            <button type="button" className="flex w-full items-center gap-3 text-left" aria-expanded={open === booking.id} onClick={() => setOpen(open === booking.id ? null : booking.id)}>
              <span className="w-24 shrink-0 text-small text-fg-muted tabular-nums">{formatIsoDay(booking.day)}</span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-body">{bookingWho(booking.party, booking.otherSide, booking.kind)}</span>
                <span className="block truncate text-small text-fg-muted">
                  {booking.movement ? `Umbuchung${booking.movement.other ? ` mit ${booking.movement.other}` : ""}` : booking.purpose || booking.kind}
                  {booking.documents.length > 0 ? ` · ${booking.documents.map((document) => document.name).join(", ")}` : ""}
                </span>
              </span>
              <span className={cn("shrink-0 tabular-nums", booking.incoming ? "text-ok" : "text-fg")}>{booking.amount}</span>
            </button>
            {open === booking.id ? (
              <div className="flex flex-col gap-2 pl-27 text-small">
                {account === null ? <p className="text-fg-muted">Konto: {booking.account}</p> : null}
                <p className="text-fg-muted">
                  {booking.kind}
                  {booking.purpose ? ` · ${booking.purpose}` : ""}
                </p>
                <div className="flex flex-wrap items-end gap-2">
                  <SelectField
                    label={booking.categoryByParty ? "Kategorie (von der Partei)" : "Kategorie"}
                    value={booking.category === null ? "" : String(booking.category)}
                    onChange={(value) => void choose(booking, value, false)}
                    options={[{ value: "", label: "Keine" }, ...categories.map((row) => ({ value: String(row.id), label: row.name }))]}
                  />
                  {booking.party && booking.category !== null ? (
                    <Button size="small" kind="plain" onClick={() => void choose(booking, String(booking.category), true)}>
                      Für alle von {booking.party}
                    </Button>
                  ) : null}
                </div>
                {booking.documents.map((document) => (
                  <div key={document.link} className="flex items-center justify-between gap-2">
                    <span>
                      Bezahlt: <span className="font-medium">{document.name}</span> <span className="text-fg-muted">— Warum? {document.why}</span>
                    </span>
                    <Button size="small" kind="plain" onClick={() => void commands.undoLink(document.link).then(() => load(0))}>
                      Rückgängig
                    </Button>
                  </div>
                ))}
                {booking.movement ? (
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-fg-muted">
                      Warum Umbuchung? {booking.movement.why}
                      {booking.movement.made ? " Die Gegenbuchung hat das Archiv angelegt." : ""}
                    </span>
                    <Button size="small" kind="plain" onClick={() => void commands.undoMove(booking.movement?.pair ?? 0).then(() => load(0))}>
                      Rückgängig
                    </Button>
                  </div>
                ) : null}
                {typeof booking.statement === "number" ? (
                  <PageImage document={booking.statement} page={booking.page ?? 1} width={640} label="Seite des Kontoauszugs" className="max-w-md rounded-control" />
                ) : null}
              </div>
            ) : null}
          </li>
        ))}
      </ul>
      {page && rows.length < page.total ? (
        <div>
          <Button onClick={() => void load(rows.length)}>Weitere laden</Button>
        </div>
      ) : null}
    </section>
  );
}
