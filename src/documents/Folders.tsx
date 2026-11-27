import { useEffect, useState } from "react";
import { type DownloadsState, type FolderRow, commands } from "../bindings";
import { ConfirmRemove, Page, Panel } from "../settings/Page";
import { Button } from "../ui";
import { downloadsLine } from "./describe";

// "Ordner" of #27, a page of Einstellungen since #49: the watched folders with the inbox first. A file saved into one
// of them comes into the archive by itself, nothing in them is moved or deleted.
export function FoldersPage() {
  const [removing, setRemoving] = useState<FolderRow | null>(null);
  const [folders, setFolders] = useState<FolderRow[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloads, setDownloads] = useState<DownloadsState | null>(null);

  useEffect(() => {
    void commands.listFolders().then((result) => {
      if (result.status === "ok") setFolders(result.data);
    });
    void commands.downloadsState().then((result) => {
      if (result.status === "ok") setDownloads(result.data);
    });
  }, []);

  const switchDownloads = async (call: () => ReturnType<typeof commands.downloadsState>) => {
    const result = await call();
    if (result.status === "ok") setDownloads(result.data);
    else setError(result.error.message);
    const listed = await commands.listFolders();
    if (listed.status === "ok") setFolders(listed.data);
  };

  const run = async (call: () => ReturnType<typeof commands.listFolders>) => {
    const result = await call();
    if (result.status === "ok") {
      setFolders(result.data);
      setError(null);
    } else setError(result.error.message);
  };

  return (
    <Page
      actions={
        <Button kind="primary" icon="plus" onClick={() => void run(() => commands.addFolder())}>
          Ordner hinzufügen …
        </Button>
      }
    >
      <Panel>
      <ul className="flex flex-col">
        {folders.map((folder) => (
          <li key={folder.id} className="flex min-h-14 items-center justify-between gap-3 border-b border-line py-2 last:border-b-0">
            <span className="min-w-0">
              <span className="block truncate text-body">{folder.path}</span>
              <span className={folder.problem ? "text-small text-danger" : "text-small text-fg-muted"}>
                {folder.problem ?? (folder.isInbox ? "Eingang, von Anfang an beobachtet" : "beobachtet, mit Unterordnern")}
              </span>
            </span>
            <span className="flex shrink-0 gap-1">
              <Button size="small" kind="plain" icon="folder" onClick={() => void commands.openFolder(folder.id)}>
                Öffnen
              </Button>
              {folder.isInbox ? (
                <Button size="small" kind="plain" onClick={() => void run(() => commands.moveInbox())}>
                  Verlegen …
                </Button>
              ) : (
                <Button size="small" kind="plain" icon="trash" onClick={() => setRemoving(folder)}>
                  Entfernen
                </Button>
              )}
            </span>
          </li>
        ))}
      </ul>
      </Panel>
      {downloads ? (
        <div className="flex flex-col gap-1.5 rounded-panel bg-surface px-4 py-3 shadow-panel">
          <div className="flex items-center justify-between gap-3">
            <span className="text-body font-medium">Downloads beobachten</span>
            <Button size="small" kind={downloads.on ? "plain" : "secondary"} onClick={() => void switchDownloads(() => commands.setDownloads(!downloads.on))}>
              {downloads.on ? "Ausschalten" : "Einschalten"}
            </Button>
          </div>
          <p className="text-small text-fg-muted">{downloadsLine(downloads)}</p>
          {downloads.on && downloads.lying > 0 ? (
            <div>
              <Button size="small" kind="plain" onClick={() => void switchDownloads(() => commands.recheckDownloads())}>
                Vorhandene prüfen
              </Button>
            </div>
          ) : null}
        </div>
      ) : null}
      {error ? <p className="text-small text-danger">{error}</p> : null}
      <ConfirmRemove
        open={removing !== null}
        title="Ordner nicht mehr beobachten?"
        goes={["Das Beobachten dieses Ordners"]}
        stays={["Die Dateien im Ordner", "Die Dokumente im Archiv"]}
        onCancel={() => setRemoving(null)}
        onRemove={() => {
          const folder = removing;
          setRemoving(null);
          if (folder) void run(() => commands.removeFolder(folder.id));
        }}
      />
    </Page>
  );
}
