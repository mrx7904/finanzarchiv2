import { useCallback, useEffect, useState } from "react";
import { type DocumentRow_Serialize as DocumentRow, type ImportSummary_Serialize as ImportSummary, commands, events } from "../bindings";
import { describeImport, formatDay, kindLabel } from "./describe";

// The list of #04 in the placeholder look of #01, until the design system of #07 replaces it.
export function Documents() {
  const [rows, setRows] = useState<DocumentRow[]>([]);
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

  const line = last ? describeImport(last) : null;
  return (
    <section className="flex w-full max-w-3xl flex-col gap-3">
      <div className="flex items-center gap-2">
        <button type="button" disabled={busy} onClick={() => void pick("files")} className="rounded-md bg-neutral-800 px-3 py-1.5 text-sm text-white disabled:opacity-50 dark:bg-neutral-200 dark:text-neutral-900">
          Dateien importieren …
        </button>
        <button type="button" disabled={busy} onClick={() => void pick("folder")} className="rounded-md border border-neutral-300 px-3 py-1.5 text-sm disabled:opacity-50 dark:border-neutral-700">
          Ordner importieren …
        </button>
        <span className="text-xs text-neutral-500">oder Dateien und Ordner ins Fenster ziehen</span>
      </div>
      {line ? (
        <div className="text-sm">
          <p>{line.text}</p>
          {line.refused.map((entry) => (
            <p key={entry} className="text-xs text-red-600 dark:text-red-400">
              {entry}
            </p>
          ))}
        </div>
      ) : null}
      {rows.length === 0 ? (
        <p className="text-sm text-neutral-500">Noch keine Dokumente im Archiv.</p>
      ) : (
        <table className="w-full text-left text-sm">
          <thead className="text-xs text-neutral-500">
            <tr>
              <th className="py-1 font-normal">Name</th>
              <th className="py-1 font-normal">Art</th>
              <th className="py-1 font-normal">Importiert</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.id} className="border-t border-neutral-200 dark:border-neutral-800">
                <td className="py-1">{row.name}</td>
                <td className="py-1">{kindLabel(row.kind)}</td>
                <td className="py-1 tabular-nums">{formatDay(row.importedAt)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
