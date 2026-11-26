import { useEffect, useState } from "react";
import { type FolderRow, commands } from "../bindings";
import { Button, Dialog } from "../ui";

// "Ordner" of #27: the watched folders with the inbox first. A file saved into one of them comes
// into the archive by itself, nothing in them is moved or deleted.
export function Folders({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [folders, setFolders] = useState<FolderRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setError(null);
    void commands.listFolders().then((result) => {
      if (result.status === "ok") setFolders(result.data);
    });
  }, [open]);

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
      {error ? <p className="mt-2 text-small text-danger">{error}</p> : null}
    </Dialog>
  );
}
