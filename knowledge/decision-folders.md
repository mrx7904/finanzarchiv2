# Decision: watched folders and the inbox folder

**Decided:** 2026-11-26 (#27, FA-32; #28, FA-33), the inbox in the documents folder on the owner's word · applies to `src-tauri/src/folders.rs`, `src-tauri/src/downloads.rs`, the tables `watched_folders`, `seen_files`, `deleted_sources` (migrations 0016, 0017), the source `folder`

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

## The downloads folder (#28)

- **Off** until the owner switches it on in "Ordner". Then the downloads folder of the system is a watched folder (`is_downloads`), only at its top level, since its subfolders hold unpacked archives and programs rather than documents.
- **Only documents of money are taken.** `downloads::select` looks at a settled file before the intake sees it, outside the archive's lock:
  - A CSV is taken, since only a statement in Layout B is detected as one.
  - A PDF with text is taken when the signals of `kinds.rs` say invoice, payslip, receipt or statement. Where they say nothing, the local model is asked for the kind alone, from at most 80 lines. Without a model it stays out.
  - A PDF without text, an image or any other type stays out. A scan or a photo in the downloads is rarely a document of money, and the model cannot judge it without vision.
- The verdict is a German sentence in `seen_files.verdict`. A taken file shows it in the preview of its document ("Aus den Downloads übernommen (sieht aus wie eine Rechnung)."). A file left out counts as seen, so it is not looked at again until it changes.
- **What lay there is left alone.** Switching on notes each file there as `lag schon da`. "Vorhandene prüfen" forgets those notes, so the next round looks at them with the same rule. Switching off forgets the folder and its notes.

## Measured

Tests with a temporary folder:
- a file that lay there while the app was closed is taken once it stands still, and never again after a start;
- nothing in the folder is changed;
- a deleted document is refused from a folder and taken by hand.

On the owner's downloads folder, read only, counts only (2026-11-26, `measure --reader downloads`, gemma4:12b-it-qat, model unloaded afterwards):

| Files at the top level | Taken | Left out by the model | Other type | Images | Time |
|---|---|---|---|---|---|
| 95 | 1 (by code) | 20 | 68 | 6 | 72 s |

The 20 left out by the model were not looked at one by one. Whether an invoice is among them shows once the owner switches the downloads on and runs "Vorhandene prüfen".
