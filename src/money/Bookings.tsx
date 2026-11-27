import { useCallback, useEffect, useState } from "react";
import { type BookingPage, type BookingRow, type CategoryRow, commands } from "../bindings";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { BarActions, Button, DenseRow, Dialog, FilterMark, GroupHead, List, LoadingRows, Menu, SelectField, TextField } from "../ui";
import type { BookingJump } from "../navigation";
import { BookingDetail } from "./BookingDetail";
import { amountText, bookingFilter, bookingWho, formatIsoDay, monthGroups } from "./describe";

// "Umsätze" as designed (#48, knowledge/decision-views.md): dense rows grouped by month with the month's sums, newest
// first, the search, the category and the period in the head and one line with the count and the sums. The detail of
// the chosen booking stands beside the list. Money in carries a "+" and is not green (rule 13). A text a jump set
// stands as a mark with "×" (rule 9).

type BookingsProps = {
  account: { id: number; name: string } | null;
  onOpenDocument: (document: number) => void;
  /** A text a jump set, such as a party from the quick search. */
  text?: string;
  /** A period or a category a figure of Auswertungen set (#38). */
  filter?: BookingJump;
  onClearText?: () => void;
  onRenamed?: (name: string) => void;
};

export function Bookings({ account, onOpenDocument, text: jumped = "", filter, onClearText, onRenamed }: BookingsProps) {
  const [year, setYear] = useState("");
  const [years, setYears] = useState<number[]>([]);
  const [category, setCategory] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState<BookingPage | null>(null);
  const [rows, setRows] = useState<BookingRow[] | null>(null);
  const [categories, setCategories] = useState<CategoryRow[]>([]);
  const [chosen, setChosen] = useState<number | null>(null);
  const [renaming, setRenaming] = useState(false);
  const text = jumped || search;

  const load = useCallback(
    async (offset: number) => {
      const from = filter?.from ?? (year ? `${year}-01-01` : "");
      const to = filter?.to ?? (year ? `${year}-12-31` : "");
      const chosenCategory = filter?.category !== undefined ? String(filter.category) : category;
      const result = await commands.listBookings(bookingFilter(account?.id ?? null, from, to, chosenCategory, text, offset));
      if (result.status !== "ok") return;
      setPage(result.data);
      setRows((before) => (offset === 0 ? result.data.rows : [...(before ?? []), ...result.data.rows]));
    },
    [account, year, category, text, filter],
  );
  useEffect(() => {
    void load(0);
  }, [load]);
  useEffect(() => {
    void commands.listCategories().then((result) => {
      if (result.status === "ok") setCategories(result.data);
    });
    void commands.evaluationYears().then((result) => {
      if (result.status === "ok") setYears(result.data);
    });
  }, []);
  useIntakeChanged(() => void load(0));

  const choose = async (booking: BookingRow, value: string, forParty: boolean) => {
    await commands.chooseCategory(booking.id, value === "" ? null : Number(value), forParty);
    void load(0);
  };
  const selected = rows?.find((row) => row.id === chosen) ?? null;

  return (
    <>
      {account && (
        <BarActions>
          <Menu align="end" entries={[{ label: "Konto umbenennen …", onSelect: () => setRenaming(true) }]} trigger={(props) => <Button {...props} kind="plain" icon="more" label="Mehr" />} />
        </BarActions>
      )}
      <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(300px,380px)] gap-4 px-6 pt-4 pb-6">
        <section aria-label="Umsätze" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
          <div className="flex flex-col gap-2 border-b border-line p-3">
            <div className="grid grid-cols-[minmax(0,1fr)_180px_140px] gap-2">
              <TextField label="Suchen" value={search} onChange={setSearch} icon="search" clearable placeholder="Partner oder Zweck" />
              <SelectField label="Kategorie" value={category} onChange={setCategory} options={[{ value: "", label: "Jede Kategorie" }, ...categories.map((row) => ({ value: String(row.id), label: row.name }))]} />
              <SelectField label="Zeitraum" value={year} onChange={setYear} options={[{ value: "", label: "Jede Zeit" }, ...years.map((value) => ({ value: String(value), label: String(value) }))]} />
            </div>
            <div className="flex items-center gap-2">
              {jumped && <FilterMark label={jumped} onClear={() => onClearText?.()} />}
              {filter && <FilterMark label={filter.label} onClear={() => onClearText?.()} />}
              <span className="text-small text-fg-muted tabular-nums">{page ? `${page.total.toLocaleString("de-DE")} Buchungen · +${page.income} · ${page.spending}` : ""}</span>
            </div>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto">
            {rows === null ? (
              <LoadingRows loading density="dense" rows={12} />
            ) : (
              <List label="Umsätze" className="gap-0">
                {monthGroups(rows).map((group) => (
                  <div key={group.key} role="group" aria-label={group.title}>
                    <GroupHead
                      title={group.title}
                      figures={
                        <>
                          {group.income > 0 && <span>{amountText(group.income, true)}</span>}
                          {group.spending > 0 && <span>{amountText(group.spending, false)}</span>}
                        </>
                      }
                    />
                    {group.rows.map((booking) => (
                      <DenseRow
                        key={booking.id}
                        lead={formatIsoDay(booking.day).slice(0, 6)}
                        title={bookingWho(booking.party, booking.otherSide, booking.kind)}
                        detail={booking.movement ? "Umbuchung" : booking.purpose || booking.kind}
                        figure={`${booking.incoming ? "+" : ""}${booking.amount}`}
                        selected={booking.id === chosen}
                        onSelect={() => setChosen(booking.id)}
                      />
                    ))}
                  </div>
                ))}
              </List>
            )}
            {page && rows && rows.length < page.total && (
              <div className="p-3">
                <Button onClick={() => void load(rows.length)}>Weitere laden</Button>
              </div>
            )}
          </div>
        </section>
        <section aria-label="Buchung" className="flex min-h-0 flex-col gap-3 overflow-y-auto rounded-panel bg-surface p-4 shadow-panel">
          {selected ? (
              <BookingDetail
                key={selected.id}
                booking={selected.id}
                categories={categories}
                category={selected.category ?? null}
                categoryByParty={selected.categoryByParty}
                onCategory={(value, forParty) => void choose(selected, value, forParty)}
                onChanged={() => void load(0)}
                onOpenDocument={onOpenDocument}
              />
          ) : (
            <p className="text-fg-muted">Keine Buchung gewählt</p>
          )}
        </section>
      </div>
      {account && <RenameAccount open={renaming} account={account} onClose={() => setRenaming(false)} onRenamed={(name) => onRenamed?.(name)} />}
    </>
  );
}

/** "Konto umbenennen …": the name only, bank and IBAN stay (decided by Fable 5.2 for the owner). */
function RenameAccount({ open, account, onClose, onRenamed }: { open: boolean; account: { id: number; name: string }; onClose: () => void; onRenamed: (name: string) => void }) {
  const [name, setName] = useState(account.name);
  const [error, setError] = useState<string | undefined>();
  useEffect(() => {
    if (open) {
      setName(account.name);
      setError(undefined);
    }
  }, [open, account.name]);
  const save = async () => {
    const result = await commands.renameAccount(account.id, name);
    if (result.status === "ok") {
      onClose();
      onRenamed(name.trim());
    } else setError(result.error.message);
  };
  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="narrow"
      title="Wie soll das Konto heißen?"
      actions={
        <>
          <Button onClick={onClose}>Abbrechen</Button>
          <Button kind="primary" onClick={() => void save()}>
            Umbenennen
          </Button>
        </>
      }
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void save();
        }}
      >
        <TextField label="Name" value={name} onChange={setName} hint="Bank und IBAN bleiben." error={error} />
      </form>
    </Dialog>
  );
}
