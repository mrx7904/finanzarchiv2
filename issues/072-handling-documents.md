# #72 Handling documents: the drop, the keys, several at once

**Goal:** The owner sees where a dropped file lands, moves through the lists by the keyboard, opens a document in the system's program and acts on several documents at once.
**Requirements**: FA-72 · **Depends on**: #46, #48, #61

Found by reading the code on 2026-11-27. A file dragged over the window gets no sign that it may be dropped: the drop is taken in the core (`lib.rs`) and the window draws nothing. A document can be shown in its folder but not opened. Dokumente, "Umsätze" and "Bestellungen" choose one row at a time, so deleting ten wrong imports is ten dialogs. 1.0 had keys for almost every action.

## Done when

- [ ] While files are dragged over the window, every view shows one calm mark "Hier ablegen", and after the drop the line of the intake says how many files were taken. A file that is refused is named there with its reason, as today.
- [ ] The menu of a document has "Öffnen" (Enter on the chosen row): the original opens in the program of the system. To verify: the opener plugin of Tauri 2 and the narrowest permission that allows only files inside the archive folder, at v2.tauri.app on the day of the work.
- [ ] In every list with detail the arrow keys move the choice, Pos1 and Ende jump to the ends, and the list scrolls with it. "Entf" asks to delete the chosen document. Strg+F (⌘F) puts the cursor into the search of the view.
- [ ] Einstellungen › Über lists every key of the app, from one table in the code that the handlers read too, so the list cannot go stale.
- [ ] In Dokumente, Strg+click and Umschalt+click choose several rows. The detail then shows the count and the actions that fit all of them: "Löschen …", "Für die Steuer …" and "Art ändern …" (#61). The dialog of deleting names the count and what goes.
- [ ] In "Umsätze" several chosen bookings take one category at once.
- [ ] An action on several rows is one decision that "Rückgängig" undoes as a whole, where the single action can be undone.
- [ ] Tests cover the choice by Strg and Umschalt, the keys of the lists and the table of the keys. `npm run check:views` shows the drop mark in light and dark with 0 findings.

## Out of scope

- Dragging a document out of the window.
- Printing.
