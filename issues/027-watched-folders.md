# #27 Watched folders and the inbox folder

**Goal:** A file saved into a watched folder is in the archive without a click, and an inbox folder is there from the start.
**Requirements**: FA-32 · **Depends on**: #05

## Done when

- [ ] `watched_folders` with path, whether it is the inbox and whether subfolders count. The inbox folder "Finanzarchiv Eingang" is made on the first start in the documents folder of the system (the owner, 2026-11-26) and can be moved, not removed.
- [ ] A watcher of the system (`notify`) sees new and changed files. A file is taken once its size has stood still, and goes through the intake as a drop does, with the folder as its source. At every start the folders are read again, so a file saved while the app was closed is found.
- [ ] Nothing in a folder is moved, renamed or deleted. A file deleted from the archive is not taken again from its folder. A folder that is gone or does not answer is shown, never a failure.
- [ ] Tested with a temporary folder: a new file, a file still being written, a restart, a file deleted from the archive. "Ordner" in the window lists the folders, with "Hinzufügen", "Entfernen" and "Öffnen".
