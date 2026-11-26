# Decision: watched folders and the inbox folder

**Decided:** 2026-11-26 (#27, FA-32), the inbox in the documents folder on the owner's word · applies to `src-tauri/src/folders.rs`, the tables `watched_folders`, `seen_files`, `deleted_sources` (migration 0016), the source `folder`

## Decision

- **The inbox folder** "Finanzarchiv Eingang" is made on the first start in the documents folder of the system (the owner chose Dokumente on 2026-11-26, as in 1.0), and made again at a start when it was deleted. It can be moved ("Verlegen …"), never removed. The old place is then no longer watched, and its files stay archived.
- **Watched folders** are added with "Ordner hinzufügen …" in "Ordner" (from "Dokumente"), with their subfolders, and can be removed. A folder inside the archive's own folder is refused, since it would read the archive back into itself.
- **Watching:**
  - A watcher of the system (`notify` 9.0.0) notes new and changed files.
  - A round every 0.7 s hands a file to the intake once its size and time of change have stood still for 2 s. A download still running (`.crdownload`, `.part` and the like), a hidden file or a system file is never taken.
  - At every start, and after every change of the list, each folder is read once, so a file saved while the app was closed is found.
  - A file handed in is a row of `seen_files` (path, size, time of change), so a start never hands it again, while a changed file is handed again (the intake knows its hash).
- A file comes in with the source `folder`. `documents` and `intake_jobs` were built anew to allow `folder` and `mail` (migration 0016). Nothing in a folder is moved, renamed or deleted, a document's original is a copy under its hash.
- **Deleted stays deleted.** A document the owner deletes leaves its hash in `deleted_sources`, and a watched folder or a mailbox does not bring it back ("Diese Datei hast du aus dem Archiv gelöscht …"). Taking it in by hand still works.
- A folder that is gone or does not answer is shown under its path ("Der Ordner ist nicht erreichbar."), never a failure of the app. "Öffnen" opens a watched folder in the file manager of the system, by its id, so the window can open no other path.

## Measured

Tests with a temporary folder:
- a file that lay there while the app was closed is taken once it stands still, and never again after a start;
- nothing in the folder is changed;
- a deleted document is refused from a folder and taken by hand.
