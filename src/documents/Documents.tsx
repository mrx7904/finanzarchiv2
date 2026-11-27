import { useCallback, useEffect, useState } from "react";
import { type DocumentRow_Serialize as DocumentRow, commands } from "../bindings";
import { queuedLine } from "../intake/describe";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { Button, EmptyState, List, ListRow } from "../ui";
import { formatDay, kindLabel, stateLabel, textStateLabel } from "./describe";
import { Filing } from "./Filing";
import { MailMark, MailText } from "./MailText";
import { Orders } from "./Orders";
import { DocumentName } from "./Names";
import { PageImage } from "./PageImage";

// The documents of #04 in the pieces of the design system (#07), imports through the queue of #05,
// the first page of each and the preview of the chosen one from #06.
type DocumentsProps = {
  focus?: number | null;
  /** The owner chose a document in the list: the view keeps it as its place (#44). */
  onChoose?: (document: number | null) => void;
  /** A jump to another document, such as the paired receipt, that the arrow back leads from (#44). */
  onJump?: (document: number) => void;
};

export function Documents({ focus = null, onChoose, onJump }: DocumentsProps) {
  const [rows, setRows] = useState<DocumentRow[]>([]);
  const [chosen, setChosenHere] = useState<number | null>(focus);
  useEffect(() => {
    if (focus !== null) setChosenHere(focus);
  }, [focus]);
  const setChosen = (document: number | null) => {
    setChosenHere(document);
    onChoose?.(document);
  };
  const [queued, setQueued] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [orders, setOrders] = useState(false);

  const reload = useCallback(async () => {
    const listed = await commands.listDocuments();
    if (listed.status === "ok") setRows(listed.data);
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

  const actions = (
    <div className="flex items-center gap-2">
      <Button kind="primary" icon="upload" busy={busy} onClick={() => void pick("files")}>
        Dateien importieren …
      </Button>
      <Button icon="folder" disabled={busy} onClick={() => void pick("folder")}>
        Ordner importieren …
      </Button>
      <Button kind="plain" onClick={() => setOrders(true)}>
        Bestellungen …
      </Button>
      <Orders
        open={orders}
        onClose={() => setOrders(false)}
        onOpenDocument={(document) => {
          setOrders(false);
          setChosen(document);
        }}
      />
    </div>
  );
  const selected = rows.find((row) => row.id === chosen) ?? null;
  return (
    <section className="flex w-full max-w-5xl flex-col gap-3">
      {rows.length > 0 ? actions : null}
      {queued ? <p className="text-body text-fg-muted">{queued}</p> : null}
      {rows.length === 0 ? (
        <EmptyState icon="document" title="Noch keine Dokumente im Archiv" hint="Ziehe Dateien oder Ordner ins Fenster oder wähle sie aus." actions={actions} />
      ) : (
        <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-start gap-4">
          <List label="Dokumente">
            {rows.map((row) => (
              <ListRow
                key={row.id}
                title={row.name}
                detail={`${kindLabel(row.kind)}, ${stateLabel(row.state)}, importiert am ${formatDay(row.importedAt)}`}
                leading={
                  row.fileType === "mail" ? (
                    <MailMark className="h-10 w-8 rounded-[3px] shadow-panel" />
                  ) : (
                    <PageImage document={row.id} width={160} label={`Erste Seite von ${row.name}`} className="h-10 w-8 rounded-[3px] shadow-panel" />
                  )
                }
                selected={chosen === row.id}
                onSelect={() => setChosen(row.id)}
              />
            ))}
          </List>
          <aside aria-label="Vorschau" className="sticky top-0 flex flex-col gap-2 rounded-panel bg-surface p-3 shadow-panel">
            {selected ? (
              <>
                {selected.fileType === "mail" ? (
                  <MailText key={selected.id} document={selected.id} />
                ) : (
                  <PageImage key={selected.id} document={selected.id} width={640} label={`Erste Seite von ${selected.name}`} className="rounded-control" />
                )}
                <DocumentName document={selected.id} name={selected.name} originalName={selected.originalName} byOwner={selected.nameByOwner} onChanged={() => void reload()} />
                <p className="text-small text-fg-muted">{textStateLabel(selected.textState, selected.pages)}</p>
                {selected.arrival ? <p className="text-small text-fg-muted">{selected.arrival}</p> : null}
                {selected.paired ? (
                  <p className="text-small text-fg-muted">
                    {selected.paired.invoice ? "Zahlungsbeleg der Rechnung " : "Bezahlt über PayPal, Beleg "}
                    <button type="button" className="text-fg-accent underline-offset-2 hover:underline" onClick={() => {
                        const paired = selected.paired?.document;
                        if (paired === undefined) return;
                        if (onJump) onJump(paired);
                        else setChosen(paired);
                      }}>
                      {selected.paired.name}
                    </button>
                  </p>
                ) : null}
                <Filing key={`${selected.id}-${selected.state}`} document={selected.id} onChanged={() => void reload()} />
              </>
            ) : (
              <p className="px-2 py-8 text-center text-small text-fg-muted">Wähle ein Dokument, um seine erste Seite zu sehen.</p>
            )}
          </aside>
        </div>
      )}
    </section>
  );
}
