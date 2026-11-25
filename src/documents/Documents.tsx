import { useCallback, useEffect, useState } from "react";
import { type DocumentRow_Serialize as DocumentRow, type ImportSummary_Serialize as ImportSummary, commands, events } from "../bindings";
import { Button, EmptyState, Icon, List, ListRow } from "../ui";
import { describeImport, formatDay, kindLabel } from "./describe";

// The documents of #04 in the pieces of the design system (#07). The view of its own comes with stage 2.
export function Documents() {
  const [rows, setRows] = useState<DocumentRow[]>([]);
  const [chosen, setChosen] = useState<number | null>(null);
  const [last, setLast] = useState<ImportSummary | null>(null);
  const [busy, setBusy] = useState(false);

  const reload = useCallback(async () => {
    const listed = await commands.listDocuments();
    if (listed.status === "ok") setRows(listed.data);
  }, []);

  const finish = useCallback(
    (summary: ImportSummary) => {
      setLast(summary);
      void reload();
    },
    [reload],
  );

  useEffect(() => {
    void reload();
    const stop = events.importFinished.listen((event) => {
      const summary = event.payload.summary;
      finish({ ...summary, problem: summary.problem ?? null });
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [reload, finish]);

  const pick = async (which: "files" | "folder") => {
    setBusy(true);
    const result = which === "files" ? await commands.importFiles() : await commands.importFolder();
    setBusy(false);
    if (result.status === "ok") finish(result.data);
  };

  const actions = (
    <div className="flex items-center gap-2">
      <Button kind="primary" icon="upload" busy={busy} onClick={() => void pick("files")}>
        Dateien importieren …
      </Button>
      <Button icon="folder" disabled={busy} onClick={() => void pick("folder")}>
        Ordner importieren …
      </Button>
    </div>
  );
  const line = last ? describeImport(last) : null;
  return (
    <section className="flex w-full max-w-3xl flex-col gap-3">
      {rows.length > 0 ? actions : null}
      {line ? (
        <div className="text-body">
          <p>{line.text}</p>
          {line.refused.map((entry) => (
            <p key={entry} className="text-small text-danger">
              {entry}
            </p>
          ))}
        </div>
      ) : null}
      {rows.length === 0 ? (
        <EmptyState icon="document" title="Noch keine Dokumente im Archiv" hint="Ziehe Dateien oder Ordner ins Fenster oder wähle sie aus." actions={actions} />
      ) : (
        <List label="Dokumente">
          {rows.map((row) => (
            <ListRow
              key={row.id}
              title={row.name}
              detail={`${kindLabel(row.kind)}, importiert am ${formatDay(row.importedAt)}`}
              leading={<Icon name="document" />}
              selected={chosen === row.id}
              onSelect={() => setChosen(row.id)}
            />
          ))}
        </List>
      )}
    </section>
  );
}
