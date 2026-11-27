import { useCallback, useEffect, useState } from "react";
import { type DocumentDetail, type DocumentRow_Serialize as DocumentRow, commands } from "../bindings";
import { queuedLine } from "../intake/describe";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { BarActions, Button, Consequences, Dialog, EmptyState, Icon, List, ListRow, LoadingRows, Menu, SelectField, TextField, ValueRow, type ValueSource } from "../ui";
import { countsLine, filterRows, kindLabel, rowDetail, rowState } from "./describe";
import { Filing } from "./Filing";
import { MailMark, MailText } from "./MailText";
import { AssignTax } from "../taxes/Taxes";
import { PageImage } from "./PageImage";

// "Dokumente" as designed (#46, knowledge/decision-views.md): the list with its search, kind and state at the left,
// the page of the chosen document, and its values with where each stands, the booking that paid it, "Warum?" and
// "Rückgängig". Three columns from 1300 px, below that the values stand under the page. The top bar holds
// "Importieren" and "…"; the menu at the document holds "Umbenennen …", "Datei im Ordner zeigen" and "Löschen …".

type DocumentsProps = {
  focus?: number | null;
  /** The owner chose a document in the list: the view keeps it as its place (#44). */
  onChoose?: (document: number | null) => void;
  /** A jump to another document, such as the paired receipt, that the arrow back leads from (#44). */
  onJump?: (document: number) => void;
  /** A jump to the bookings of the account that paid it. */
  onOpenBooking?: (account: { id: number; name: string }) => void;
  /** The page "Bestellungen" below Dokumente. */
  onOrders?: () => void;
};

const kinds = ["invoice", "receipt", "statement", "payslip", "other"];
const states = ["prüfen", "bestätigt", "läuft"];

export function Documents({ focus = null, onChoose, onJump, onOpenBooking, onOrders }: DocumentsProps) {
  const [rows, setRows] = useState<DocumentRow[] | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [chosen, setChosenHere] = useState<number | null>(focus);
  const [search, setSearch] = useState("");
  const [kind, setKind] = useState("");
  const [state, setState] = useState("");
  const [queued, setQueued] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (focus !== null) setChosenHere(focus);
  }, [focus]);
  const setChosen = (document: number | null) => {
    setChosenHere(document);
    onChoose?.(document);
  };

  const reload = useCallback(async () => {
    const listed = await commands.listDocuments();
    if (listed.status === "ok") {
      setRows(listed.data);
      setFailed(null);
    } else setFailed(listed.error.message);
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);
  useIntakeChanged(() => void reload());

  const pick = async (which: "files" | "folder") => {
    setBusy(true);
    const result = which === "files" ? await commands.importFiles() : await commands.importFolder();
    setBusy(false);
    setQueued(result.status === "ok" ? queuedLine(result.data.files) : result.error.message);
  };

  const barActions = (
    <BarActions>
      <Menu
        align="end"
        entries={[
          { label: "Dateien auswählen …", icon: "upload", onSelect: () => void pick("files") },
          { label: "Ordner auswählen …", icon: "folder", onSelect: () => void pick("folder") },
        ]}
        trigger={(props) => (
          <Button {...props} kind="primary" icon="plus" busy={busy}>
            Importieren
          </Button>
        )}
      />
      {onOrders && (
        <Menu align="end" entries={[{ label: "Bestellungen", icon: "receipt", onSelect: onOrders }]} trigger={(props) => <Button {...props} kind="plain" icon="more" label="Mehr" />} />
      )}
    </BarActions>
  );

  if (failed) {
    return (
      <>
        {barActions}
        <div className="flex flex-col items-start gap-3 px-6 pt-6">
          <p className="text-danger">{failed}</p>
          <Button onClick={() => void reload()}>Erneut versuchen</Button>
        </div>
      </>
    );
  }
  if (rows !== null && rows.length === 0) {
    return (
      <>
        {barActions}
        <div className="flex flex-1 items-center justify-center">
          <EmptyState
            icon="document"
            title="Noch keine Dokumente"
            hint="Ziehe Dateien ins Fenster oder wähle sie aus."
            actions={
              <Button kind="primary" icon="upload" busy={busy} onClick={() => void pick("files")}>
                Dateien auswählen …
              </Button>
            }
          />
        </div>
      </>
    );
  }

  const shown = rows ? filterRows(rows, search, kind, state) : [];
  const selected = rows?.find((row) => row.id === chosen) ?? null;
  return (
    <>
      {barActions}
      <div className="grid min-h-0 flex-1 grid-cols-[minmax(280px,360px)_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto] gap-4 px-6 pt-4 pb-6 min-[1300px]:grid-cols-[minmax(320px,400px)_minmax(0,1fr)_minmax(280px,340px)] min-[1300px]:grid-rows-1">
        <section aria-label="Dokumente" className="row-span-2 flex min-h-0 flex-col rounded-panel bg-surface shadow-panel min-[1300px]:row-span-1">
          <div className="flex flex-col gap-2 border-b border-line p-3">
            <TextField label="Suchen" value={search} onChange={setSearch} icon="search" clearable placeholder="Name" />
            <div className="grid grid-cols-2 gap-2">
              <SelectField label="Art" value={kind} onChange={setKind} options={[{ value: "", label: "Jede Art" }, ...kinds.map((value) => ({ value, label: kindLabel(value) }))]} />
              <SelectField label="Zustand" value={state} onChange={setState} options={[{ value: "", label: "Jeder Zustand" }, ...states.map((value) => ({ value, label: value }))]} />
            </div>
            <span className="text-small text-fg-muted tabular-nums">{queued ?? (rows ? countsLine(shown) : "")}</span>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto">
            {rows === null ? (
              <LoadingRows loading rows={8} />
            ) : (
              <List label="Dokumente">
                {shown.map((row) => (
                  <ListRow
                    key={row.id}
                    title={row.name}
                    detail={rowDetail(row)}
                    leading={
                      row.fileType === "mail" ? (
                        <MailMark className="h-10 w-8 rounded-[3px] shadow-panel" />
                      ) : (
                        <PageImage document={row.id} width={160} label={`Erste Seite von ${row.name}`} className="h-10 w-8 rounded-[3px] shadow-panel" />
                      )
                    }
                    state={rowState(row.state)}
                    selected={chosen === row.id}
                    onSelect={() => setChosen(row.id)}
                  />
                ))}
              </List>
            )}
          </div>
        </section>

        <section aria-label="Seite" className="flex min-h-[320px] items-center justify-center overflow-hidden rounded-panel bg-surface p-4 shadow-panel">
          {selected ? (
            selected.fileType === "mail" ? (
              <MailText key={selected.id} document={selected.id} />
            ) : (
              <PageImage key={selected.id} document={selected.id} width={900} label={`Erste Seite von ${selected.name}`} fit />
            )
          ) : (
            <p className="self-center text-fg-muted">Kein Dokument gewählt</p>
          )}
        </section>

        <section aria-label="Angaben" className="flex min-h-0 flex-col overflow-y-auto rounded-panel bg-surface shadow-panel min-[1300px]:col-start-3 min-[1300px]:row-start-1">
          {selected && (
            <Values
              key={`${selected.id}-${selected.state}`}
              row={selected}
              onChanged={() => void reload()}
              onDeleted={() => {
                setChosen(null);
                void reload();
              }}
              onJump={(document) => (onJump ? onJump(document) : setChosen(document))}
              onOpenBooking={onOpenBooking}
            />
          )}
        </section>
      </div>
    </>
  );
}

type ValuesProps = {
  row: DocumentRow;
  onChanged: () => void;
  onDeleted: () => void;
  onJump: (document: number) => void;
  onOpenBooking?: (account: { id: number; name: string }) => void;
};

function Values({ row, onChanged, onDeleted, onJump, onOpenBooking }: ValuesProps) {
  const [detail, setDetail] = useState<DocumentDetail | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [taxing, setTaxing] = useState(false);
  useEffect(() => {
    void commands.documentDetail(row.id).then((result) => {
      if (result.status === "ok") setDetail(result.data);
    });
  }, [row.id]);

  const statement = row.kind === "statement";
  const paid = (detail?.paidBy.length ?? 0) > 0;
  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="flex items-start gap-2">
        <div className="flex min-w-0 flex-1 flex-col">
          <span className="font-medium break-words">{row.name}</span>
          {row.name !== row.originalName && <span className="truncate text-small text-fg-muted">{row.originalName}</span>}
        </div>
        <Menu
          align="end"
          entries={[
            { label: "Umbenennen …", onSelect: () => setRenaming(true) },
            { label: "Für die Steuer …", icon: "scale", onSelect: () => setTaxing(true) },
            { label: "Datei im Ordner zeigen", icon: "folder", onSelect: () => void commands.revealDocument(row.id) },
            { label: "Löschen …", icon: "trash", danger: true, onSelect: () => setDeleting(true) },
          ]}
          trigger={(props) => <Button {...props} kind="plain" size="small" icon="more" label="Mehr zum Dokument" />}
        />
      </div>

      {detail && detail.values.length > 0 && (
        <div className="flex flex-col">
          {detail.values.map((value) => (
            <ValueRow key={value.field} name={value.name} value={value.value} source={value.source as ValueSource} amount={value.amount} />
          ))}
        </div>
      )}

      {detail?.paidBy.map((booking) => (
        <button
          key={booking.booking}
          type="button"
          onClick={() => onOpenBooking?.({ id: booking.account, name: booking.accountName })}
          className="flex items-center gap-2 rounded-row px-2 py-1.5 text-left transition-quick hover:bg-hover"
        >
          <Icon name="card" size={16} className="text-fg-muted" />
          <span className="flex-1 truncate">
            {booking.day.split("-").reverse().join(".")} · {booking.accountName}
          </span>
          <span className="font-medium tabular-nums">
            {booking.incoming ? "+" : ""}
            {booking.amount}
          </span>
        </button>
      ))}

      {row.paired && (
        <button type="button" onClick={() => onJump(row.paired?.document ?? row.id)} className="flex items-center gap-2 rounded-row px-2 py-1.5 text-left transition-quick hover:bg-hover">
          <Icon name="receipt" size={16} className="text-fg-muted" />
          <span className="flex-1 truncate">{row.paired.name}</span>
          <span className="text-small text-fg-muted">{row.paired.invoice ? "Rechnung" : "Beleg"}</span>
        </button>
      )}

      <Filing document={row.id} onChanged={onChanged} />

      <AssignTax open={taxing} document={row.id} onClose={() => setTaxing(false)} />
      <RenameDialog open={renaming} document={row.id} name={row.name} onClose={() => setRenaming(false)} onRenamed={onChanged} />
      <Dialog
        open={deleting}
        onClose={() => setDeleting(false)}
        size="narrow"
        title="Dokument löschen?"
        actions={
          <>
            <Button onClick={() => setDeleting(false)}>Abbrechen</Button>
            <Button
              kind="danger"
              icon="trash"
              onClick={() =>
                void commands.deleteDocument(row.id).then((result) => {
                  setDeleting(false);
                  if (result.status === "ok") onDeleted();
                })
              }
            >
              Löschen
            </Button>
          </>
        }
      >
        <Consequences
          goes={[
            statement ? "Der Kontoauszug und seine Buchungen" : "Das Dokument und seine Seiten",
            "Die gelesenen Werte",
            ...(paid ? ["Die Verknüpfung mit der Buchung"] : []),
          ]}
          stays={["Die Datei im Papierkorb des Archivs", ...(paid && !statement ? ["Die Buchung im Konto"] : [])]}
        />
      </Dialog>
    </div>
  );
}

function RenameDialog({ open, document, name, onClose, onRenamed }: { open: boolean; document: number; name: string; onClose: () => void; onRenamed: () => void }) {
  const [text, setText] = useState(name);
  const [error, setError] = useState<string | undefined>();
  useEffect(() => {
    if (open) {
      setText(name);
      setError(undefined);
    }
  }, [open, name]);
  const save = async () => {
    const result = await commands.renameDocument(document, text);
    if (result.status === "ok") {
      onClose();
      onRenamed();
    } else setError(result.error.message);
  };
  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="narrow"
      title="Wie soll das Dokument heißen?"
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
        <TextField label="Name" value={text} onChange={setText} hint="Leer: nach der Vorlage" error={error} />
      </form>
    </Dialog>
  );
}
