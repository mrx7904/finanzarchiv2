# #53 Fitting a view: widths, the rail, folded columns, the size

**Goal:** The owner gives the room of the window to what the work at hand needs, and the app keeps it that way.
**Requirements**: FA-53 · **Depends on**: #44, #45, #46, #48, #51

The rules 23 to 25 and 27: [`knowledge/decision-views.md`](../knowledge/decision-views.md), "Fitting a view". The pictures: `?v=dokumente-ziehen`, `?v=dokumente-leiste`, `?v=dokumente-ohne-angaben`, `?v=dokumente-leiste-ohne-angaben`, `?v=dokumente-zustand-waehlen`, `?v=dokumente-ohne-zustand`, `?v=umsaetze-ohne-detail`, `?v=dokumente-gross`, `?v=dokumente-klein`.

## Done when

- [x] The line between two columns can be dragged in every list with detail, by mouse and, with the focus on it, by the arrow keys. A double click gives the width of the design back. No column gets narrower than its least width, and the page of a document never narrower than 320 px: below that the values fold under the page.
- [x] The sidebar folds to a rail of 56 px by its button and by Strg+B (⌘B): icons with their names as tooltips, the count of "Prüfen" and the intake stay in sight.
- [x] The values of a document and the detail of a booking fold away by a button of the top bar that stays pressed while the column shows.
- [x] In Dokumente the filter of the states ends, below a line, with "Zustand ausblenden". Chosen, the rows show no state and give its room to the name, the filter reads "Ausgeblendet", and the line of counts still says how many wait. Any other entry of the filter shows the states again. The filter lists each state with its count.
- [x] The size of the whole view is Klein, Mittel or Groß (90, 100, 115 %), chosen in Einstellungen › Darstellung and by Strg with +, − and 0, as the zoom of the window. To be verified here at v2.tauri.app: how a webview is zoomed in Tauri 2 on WebView2 and WKWebView, and which permission it needs. The permission gets its row in [`knowledge/decision-app-shell.md`](../knowledge/decision-app-shell.md).
- [x] Widths, the rail, the folded columns and the size are kept over a restart, per view where they belong to one. Where they are kept (not in the archive, they belong to the computer) is decided here and recorded.
- [x] A view folds by the width it really has: with the size Groß a window of 1440 px behaves as one of 1252 px.
- [x] The check of #51 opens every view with the rail, with a column folded and at Klein and Groß. Tests cover the least widths and what is kept.
- [ ] Seen by the owner on Windows.

## Out of scope

- Folding sections of an overview: rejected in the decision.
- Reading a document large and the focus of "Prüfen": #54.

**Status:** built on 2026-11-27, waiting for the owner's look on Windows.

- `useColumn` (`src/ui/Columns.tsx`): the line between the list and its detail in Dokumente, Bestellungen, Umsätze, Wiederkehrend, Prüfen, Steuern and Chat, by mouse, by the arrow keys in steps of 16 px, a double click back to the design's width, held between a least and a greatest width (`clampWidth`). In Dokumente the page keeps at least 320 px.
- The rail of 56 px by the button at the top of the sidebar and Strg+B (⌘B): icons with their names as tooltips, the count of Prüfen at its corner, the intake as one cell.
- "Angaben" in the top bar of Dokumente and "Detail" in Umsätze fold their column and stay pressed while it shows.
- The filter of the states lists each with its count and ends, below a line, with "Zustand ausblenden"; chosen, it reads "Ausgeblendet", the rows show no state, and the line of counts still says how many wait.
- The sizes: `knowledge/decision-app-shell.md`, "Fitting a view", with the permission and where everything is kept.
- `npm run check:views` opens Dokumente, Umsätze, Prüfen, Auswertungen and Chat with the rail, folded, at Klein and at Groß as well: 0 findings over 131 pages. Tests: `src/ui/layout.test.ts` (least and greatest widths, what is kept, the size keys).
