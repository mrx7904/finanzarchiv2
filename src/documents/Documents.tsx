import { useCallback, useEffect, useState } from "react";
import { type DocumentRow_Serialize as DocumentRow, commands } from "../bindings";
import { queuedLine } from "../intake/describe";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { Button, EmptyState, Icon, List, ListRow } from "../ui";
import { formatDay, kindLabel } from "./describe";

// The documents of #04 in the pieces of the design system (#07). Imports go through the queue of #05.
export function Documents() {
  const [rows, setRows] = useState<DocumentRow[]>([]);
  const [chosen, setChosen] = useState<number | null>(null);
  const [queued, setQueued] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

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
    </div>
  );
  return (
    <section className="flex w-full max-w-3xl flex-col gap-3">
      {rows.length > 0 ? actions : null}
      {queued ? <p className="text-body text-fg-muted">{queued}</p> : null}
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
