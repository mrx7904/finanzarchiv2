# #71 The archive fills while the window is closed

**Goal:** Folders and mailboxes are read from the start of the computer on, without the owner opening the app.
**Requirements**: FA-71 · **Depends on**: #70

Stage 4 is done "when a document the owner saves, downloads or receives by mail is in the archive without a click". As of 0.7.0 that holds only while the window is open: the watcher and the mail reader live in the app's process (`lib.rs`), nothing starts it with the system, and closing the window ends it.

**Needs the owner's word before the work begins:** whether the app may start with the system and stay in the tray. Both are off until the owner switches them on.

## Done when

- [ ] Einstellungen gains a page "Hintergrund" with two switches, both off at the start: "Mit dem Computer starten" and "Beim Schließen weiterlaufen".
- [ ] With the first on, the app starts with the system without a window. To verify: the autostart plugin of Tauri 2 on Windows and macOS and its permission, at v2.tauri.app on the day of the work.
- [ ] With the second on, closing the window leaves an icon in the tray (the menu bar on the Mac) with "Öffnen", "Eingang pausieren" and "Beenden". The icon shows when documents wait in "Prüfen". To verify: the tray of Tauri 2 and its permission.
- [ ] Without a window the intake, the watcher and the mail reader work as with one, and the daily snapshot is still written.
- [ ] Opening the app again shows the window within a second, with what came in meanwhile.
- [ ] No notification of the system is shown (`spec/concept.md`, "Not in scope": reminders).
- [ ] The permissions get their rows in `knowledge/decision-app-shell.md`, the decision its file `knowledge/decision-background.md` with its row in `CLAUDE.md`.
- [ ] Seen on Windows: the computer restarted, a PDF saved into the inbox folder, the app opened, the document is there.

## Out of scope

- A service of the system without the app.
- Updates from inside the app: stage 6.
