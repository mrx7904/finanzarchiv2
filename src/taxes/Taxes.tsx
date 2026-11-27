import { useCallback, useEffect, useState } from "react";
import { type TaxCategoryRow, type TaxProfileRow, type TaxYearRow_Serialize as TaxYearRow, commands } from "../bindings";
import { BarActions, Button, Dialog, EmptyState, List, ListRow, LoadingRows, Segmented, SelectField, Switch, TextField } from "../ui";

// "Steuern" (#41, knowledge/decision-taxes.md) as a list with detail: the year in the top bar, the notice to check
// everything in one line, the sections by form and section at the left (never a line number), and at the right the
// entries of the chosen section, or the sums of the year with the tax profile. No tax and no refund is computed.

const day = (iso: string) => iso.split("-").reverse().join(".");

export function Taxes({ onOpenDocument, onOpenBooking }: { onOpenDocument: (document: number) => void; onOpenBooking: () => void }) {
  const [years, setYears] = useState<number[] | null>(null);
  const [year, setYear] = useState<number>(new Date().getFullYear());
  const [row, setRow] = useState<TaxYearRow | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const load = useCallback(async (which: number) => {
    const result = await commands.taxYear(which);
    if (result.status === "ok") setRow(result.data);
  }, []);
  useEffect(() => {
    void commands.taxYears().then((result) => {
      const listed = result.status === "ok" ? result.data : [];
      const now = new Date().getFullYear();
      const all = [...new Set([now, now - 1, ...listed])].sort((a, b) => b - a).slice(0, 5);
      setYears(all);
    });
  }, []);
  useEffect(() => {
    setRow(null);
    setChosen(null);
    void load(year);
  }, [year, load]);

  const section = row?.sections.find((one) => one.category.key === chosen) ?? null;
  return (
    <>
      {years && (
        <BarActions>
          <Segmented label="Steuerjahr" value={String(year)} onChange={(value) => setYear(Number(value))} options={years.map((one) => ({ value: String(one), label: String(one) }))} />
        </BarActions>
      )}
      <div className="flex min-h-0 flex-1 flex-col gap-3 px-6 pt-4 pb-6">
        <p className="text-small text-fg-muted">
          {row?.notice ?? "Keine Steuerberatung, bitte alles selbst prüfen."}
          {row && (row.valuesSource ? ` · Werte ${row.valuesSource}` : ` · Für ${year} sind keine Werte hinterlegt, Pauschalen fehlen.`)}
        </p>
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(300px,420px)_minmax(0,1fr)] gap-4">
          <section aria-label="Abschnitte" className="flex min-h-0 flex-col overflow-y-auto rounded-panel bg-surface shadow-panel">
            {row === null ? (
              <LoadingRows loading />
            ) : (
              <List label="Abschnitte">
                <ListRow title="Übersicht" detail={`Werbungskosten ${row.werbungskosten.text}`} selected={chosen === null} onSelect={() => setChosen(null)} />
                {row.sections.map((one) => (
                  <ListRow key={one.category.key} title={one.category.label} detail={one.category.elster} figure={one.counted.text} selected={chosen === one.category.key} onSelect={() => setChosen(one.category.key)} />
                ))}
              </List>
            )}
            {row && row.sections.length === 0 && <EmptyState icon="scale" title={`Für ${year} ist nichts zugeordnet`} hint="Ein Beleg kommt über „Für die Steuer …“ an ihm hierher." />}
          </section>
          <section aria-label={section ? section.category.label : "Übersicht"} className="flex min-h-0 flex-col gap-4 overflow-y-auto rounded-panel bg-surface p-4 shadow-panel">
            {row === null ? null : section ? (
              <>
                <div className="flex flex-col gap-0.5">
                  <span className="font-medium">{section.category.label}</span>
                  <span className="text-small text-fg-muted">{section.category.elster}</span>
                </div>
                <ul className="flex flex-col">
                  {section.entries.map((entry) => (
                    <li key={`${entry.document ?? ""}-${entry.booking ?? ""}`} className="border-b border-line last:border-b-0">
                      <button
                        type="button"
                        onClick={() => (entry.document !== null ? onOpenDocument(entry.document) : onOpenBooking())}
                        className="grid w-full grid-cols-[6rem_minmax(0,1fr)_7rem] items-center gap-3 rounded-row px-2 py-2 text-left transition-quick hover:bg-hover"
                      >
                        <span className="text-small text-fg-muted tabular-nums">{day(entry.day)}</span>
                        <span className="flex min-w-0 flex-col">
                          <span className="truncate">{entry.document !== null ? "Beleg" : "Buchung"} · {entry.gross.text}</span>
                          {entry.notes.length > 0 && <span className="truncate text-small text-fg-muted">{entry.notes.join(" · ")}</span>}
                        </span>
                        <span className="text-right font-medium tabular-nums">{entry.counted.text}</span>
                      </button>
                    </li>
                  ))}
                </ul>
                <span className="text-right text-small text-fg-muted tabular-nums">voraussichtlich {section.counted.text}</span>
              </>
            ) : (
              <Overview row={row} year={year} onSaved={() => void load(year)} />
            )}
          </section>
        </div>
      </div>
    </>
  );
}

function Line({ name, value, note }: { name: string; value: string; note?: string }) {
  return (
    <div className="flex min-h-10 items-center gap-3 border-b border-line py-1.5 last:border-b-0">
      <span className="flex min-w-0 flex-1 flex-col">
        <span>{name}</span>
        {note && <span className="text-small text-fg-muted">{note}</span>}
      </span>
      <span className="font-medium tabular-nums">{value}</span>
    </div>
  );
}

/** The sums of the year and the tax profile, what no document shows. */
function Overview({ row, year, onSaved }: { row: TaxYearRow; year: number; onSaved: () => void }) {
  const [profile, setProfile] = useState<TaxProfileRow>(row.profile);
  const [saved, setSaved] = useState(false);
  useEffect(() => setProfile(row.profile), [row.profile]);
  const number = (key: keyof TaxProfileRow, label: string, euros = false) => (
    <TextField
      label={label}
      inputMode="numeric"
      value={String(euros ? Number(profile[key]) / 100 : profile[key])}
      onChange={(value) => {
        const parsed = Math.max(0, Math.round(Number(value.replace(",", ".")) * (euros ? 100 : 1)) || 0);
        setProfile({ ...profile, [key]: parsed });
        setSaved(false);
      }}
    />
  );
  return (
    <div className="flex max-w-2xl flex-col gap-6">
      <div className="flex flex-col">
        <Line name="Werbungskosten" value={row.werbungskosten.text} note={row.pauschbetrag ? `Arbeitnehmer-Pauschbetrag ${row.pauschbetrag.text}` : undefined} />
        {row.abovePauschbetrag && <Line name="Über dem Pauschbetrag" value={row.abovePauschbetrag.text} />}
        {row.entfernungspauschale && <Line name="Entfernungspauschale" value={row.entfernungspauschale.text} note="Anlage N · Werbungskosten · Wege zwischen Wohnung und erster Tätigkeitsstätte" />}
        {row.homeoffice && <Line name="Homeoffice-Pauschale" value={row.homeoffice.text} note="Anlage N · Werbungskosten · Häusliches Arbeitszimmer / Tagespauschale" />}
        {row.erstausbildung.cents > 0 && <Line name="Erstausbildung" value={row.erstausbildung.text} />}
        {row.haushaltReduction && row.haushaltReduction.cents > 0 && <Line name="Steuerermäßigung § 35a Haushalt" value={row.haushaltReduction.text} />}
        {row.handwerkerReduction && row.handwerkerReduction.cents > 0 && <Line name="Steuerermäßigung § 35a Handwerker" value={row.handwerkerReduction.text} />}
      </div>
      <form
        className="flex flex-col gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          void commands.setTaxProfile(year, profile).then((result) => {
            if (result.status === "ok") {
              setSaved(true);
              onSaved();
            }
          });
        }}
      >
        <span className="text-small font-medium text-fg-muted">Steuerprofil {year}</span>
        <div className="grid grid-cols-2 gap-3">
          {number("commuteKm", "Kilometer zur Arbeit")}
          {number("commuteDays", "Tage im Büro")}
          {number("homeofficeDays", "Tage zu Hause")}
          {number("employerCommuteCents", "Zuschuss Arbeitsweg in €", true)}
        </div>
        <Switch label="Eigenes Auto" checked={profile.ownCar} onChange={(ownCar) => setProfile({ ...profile, ownCar })} />
        <div className="flex items-center gap-3">
          <Button type="submit" kind="primary">
            Sichern
          </Button>
          {saved && <span className="text-small text-fg-muted">Gesichert</span>}
        </div>
      </form>
    </div>
  );
}

/** "Für die Steuer …" at a document or a booking: its category, the share for work, what someone refunded, the labour share. */
export function AssignTax({ open, document, booking, onClose }: { open: boolean; document?: number; booking?: number; onClose: () => void }) {
  const [categories, setCategories] = useState<TaxCategoryRow[]>([]);
  const [category, setCategory] = useState("");
  const [share, setShare] = useState("100");
  const [refunded, setRefunded] = useState("0");
  const [labour, setLabour] = useState("");
  const [error, setError] = useState<string | undefined>();
  useEffect(() => {
    if (!open) return;
    setError(undefined);
    void commands.taxCategories().then(setCategories);
  }, [open]);
  const cents = (text: string) => Math.max(0, Math.round(Number(text.replace(",", ".")) * 100) || 0);
  const chosen = categories.find((one) => one.key === category);
  const save = async () => {
    const result = await commands.assignTax({
      document: document ?? null,
      booking: booking ?? null,
      category,
      sharePercent: Math.min(100, Math.max(1, Number(share) || 100)),
      refundedCents: cents(refunded),
      labourCents: labour ? cents(labour) : null,
    });
    if (result.status === "ok") onClose();
    else setError(result.error.message);
  };
  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Wofür zählt es bei der Steuer?"
      actions={
        <>
          {(document !== undefined || booking !== undefined) && (
            <Button kind="plain" onClick={() => void commands.unassignTax(document ?? null, booking ?? null).then(onClose)}>
              Keine Steuer
            </Button>
          )}
          <Button onClick={onClose}>Abbrechen</Button>
          <Button kind="primary" disabled={!category} onClick={() => void save()}>
            Zuordnen
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <SelectField label="Kategorie" value={category} onChange={setCategory} options={[{ value: "", label: "Wählen" }, ...categories.map((one) => ({ value: one.key, label: one.label }))]} />
        {chosen && <span className="text-small text-fg-muted">{chosen.elster}</span>}
        <div className="grid grid-cols-2 gap-3">
          <TextField label="Anteil beruflich in %" inputMode="numeric" value={share} onChange={setShare} />
          <TextField label="Erstattet in €" inputMode="decimal" value={refunded} onChange={setRefunded} />
          {chosen?.key === "handwerker" && <TextField label="Lohnanteil in €" inputMode="decimal" value={labour} onChange={setLabour} />}
        </div>
        {error && <p className="text-small text-danger">{error}</p>}
        <p className="text-small text-fg-muted">Keine Steuerberatung, bitte selbst prüfen.</p>
      </div>
    </Dialog>
  );
}
