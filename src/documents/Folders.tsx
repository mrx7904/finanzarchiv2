import { useEffect, useState } from "react";
import { type DownloadsState, type FolderRow, commands } from "../bindings";
import { Button, Dialog } from "../ui";
import { downloadsLine } from "./describe";

// "Ordner" of #27: the watched folders with the inbox first. A file saved into one of them comes
// into the archive by itself, nothing in them is moved or deleted.
export function Folders({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [folders, setFolders] = useState<FolderRow[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloads, setDownloads] = useState<DownloadsState | null>(null);

  useEffect(() => {
    if (!open) return;
    setError(null);
    void commands.listFolders().then((result) => {
      if (result.status === "ok") setFolders(result.data);
    });
    void commands.downloadsState().then((result) => {
      if (result.status === "ok") setDownloads(result.data);
    });
  }, [open]);

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
    <Dialog
      open={open}
      onClose={onClose}
      title="Ordner"
      description="Was du in einen dieser Ordner legst, kommt von selbst ins Archiv. Dort wird nichts verschoben oder gelöscht."
      actions={
        <>
          <Button icon="plus" onClick={() => void run(() => commands.addFolder())}>
            Ordner hinzufügen …
          </Button>
          <Button kind="primary" onClick={onClose}>
            Fertig
          </Button>
        </>
      }
    >
      <ul className="flex flex-col gap-2">
        {folders.map((folder) => (
          <li key={folder.id} className="flex items-center justify-between gap-3">
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
                <Button size="small" kind="plain" icon="trash" onClick={() => void run(() => commands.removeFolder(folder.id))}>
                  Entfernen
                </Button>
              )}
            </span>
          </li>
        ))}
      </ul>
      {downloads ? (
        <div className="mt-4 flex flex-col gap-1.5 border-t border-line pt-3">
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
      {error ? <p className="mt-2 text-small text-danger">{error}</p> : null}
    </Dialog>
  );
}
