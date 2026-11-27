# #49 The view "Einstellungen"

**Goal:** Everything the owner sets up once stands in one view, and no view carries a button for it.
**Requirements**: FA-50 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=einstellungen` and `?v=einstellungen-ordner`, `-namen`, `-kategorien`, `-modell`, `-archiv`, `-darstellung`, `-ueber`, `?v=dialog-postfach-entfernen`, `?v=dialog-ordner-entfernen`.

## Done when

- [x] "Einstellungen" at the foot of the sidebar: its pages as a list at the left, the chosen page at the right, the path "Einstellungen › <Seite>".
- [x] Ordner, Postfächer, Namen and Kategorien hold what the dialogs `Folders`, `Mailboxes`, `TemplatesDialog` and `Categories` hold today, with the same commands. The four dialogs and their buttons in Dokumente and Konten are gone.
- [x] Modell: whether the local model is reached, which one reads, its address, and "Verbindung testen". Changing `model.name` follows [`knowledge/decision-model.md`](../knowledge/decision-model.md).
- [x] Archiv: where the archive lies with its counts, and the snapshots with their day and reason. "Wiederherstellen …" and "Umziehen …" are shown only as far as the core can do them.
- [x] Darstellung: light, dark or as the system, kept over a restart. The size of the view comes with #53.
- [x] Über: the version, and the pieces with their licences from the table of [`knowledge/decision-ui-libraries.md`](../knowledge/decision-ui-libraries.md), which that file promised to move into the app.
- [x] Removing a mailbox or a folder asks with what goes and what stays (rule 18). A problem of a folder or mailbox shows on its row and in the list of pages.
- [x] Tests cover that every page is reached by the quick search in the owner's words (rule 22).
- [ ] Seen by the owner on Windows.

## Out of scope

- Moving the archive, restoring from the view, updates, profiles: stage 6.
- The size of the view (Klein, Mittel, Groß): #53. The view the app opens with: drawn, decided when the owner asks for it.

**Status:** built on 2026-11-27, waiting for the owner's look on Windows.

- The four dialogs are pages now (`FoldersPage`, `MailboxesPage`, `TemplatesPage`, `CategoriesPage`), with the same commands and their actions in the top bar. Their buttons in Dokumente and Konten are gone; "Bestellungen" stays in Dokumente until #46 makes it a page.
- Two commands are new in the core: `archive_info` (the folder, the counts, the snapshots with their reason read from their name) and `model_status` (`/api/tags` of Ollama: reached, installed, its address). The model is shown, not changed here: changing `model.name` stays as `decision-model.md` has it.
- Archiv lists the snapshots without "Wiederherstellen …" and without "Umziehen …": the core can do neither yet (stage 6).
- Darstellung keeps light, dark or as the system in the webview's storage, on the computer and not in the archive, set before the first paint.
- Tests: `src/settings/settings.test.ts` (every page found by the owner's words, the color scheme kept). **Not seen in the Tauri window**, for the reason of #44.
