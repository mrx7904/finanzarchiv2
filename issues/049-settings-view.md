# #49 The view "Einstellungen"

**Goal:** Everything the owner sets up once stands in one view, and no view carries a button for it.
**Requirements**: FA-50 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=einstellungen` and `?v=einstellungen-ordner`, `-namen`, `-kategorien`, `-modell`, `-archiv`, `-darstellung`, `-ueber`, `?v=dialog-postfach-entfernen`, `?v=dialog-ordner-entfernen`.

## Done when

- [ ] "Einstellungen" at the foot of the sidebar: its pages as a list at the left, the chosen page at the right, the path "Einstellungen › <Seite>".
- [ ] Ordner, Postfächer, Namen and Kategorien hold what the dialogs `Folders`, `Mailboxes`, `TemplatesDialog` and `Categories` hold today, with the same commands. The four dialogs and their buttons in Dokumente and Konten are gone.
- [ ] Modell: whether the local model is reached, which one reads, its address, and "Verbindung testen". Changing `model.name` follows [`knowledge/decision-model.md`](../knowledge/decision-model.md).
- [ ] Archiv: where the archive lies with its counts, and the snapshots with their day and reason. "Wiederherstellen …" and "Umziehen …" are shown only as far as the core can do them.
- [ ] Darstellung: light, dark or as the system, kept over a restart. The size of the view comes with #53.
- [ ] Über: the version, and the pieces with their licences from the table of [`knowledge/decision-ui-libraries.md`](../knowledge/decision-ui-libraries.md), which that file promised to move into the app.
- [ ] Removing a mailbox or a folder asks with what goes and what stays (rule 18). A problem of a folder or mailbox shows on its row and in the list of pages.
- [ ] Tests cover that every page is reached by the quick search in the owner's words (rule 22).
- [ ] Seen by the owner on Windows.

## Out of scope

- Moving the archive, restoring from the view, updates, profiles: stage 6.
- The size of the view (Klein, Mittel, Groß): #53. The view the app opens with: drawn, decided when the owner asks for it.
