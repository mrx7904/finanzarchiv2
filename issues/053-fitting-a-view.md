# #53 Fitting a view: widths, the rail, folded columns, the size

**Goal:** The owner gives the room of the window to what the work at hand needs, and the app keeps it that way.
**Requirements**: FA-53 · **Depends on**: #44, #45, #46, #48, #51

The rules 23 to 25 and 27: [`knowledge/decision-views.md`](../knowledge/decision-views.md), "Fitting a view". The pictures: `?v=dokumente-ziehen`, `?v=dokumente-leiste`, `?v=dokumente-ohne-angaben`, `?v=dokumente-leiste-ohne-angaben`, `?v=dokumente-zustand-waehlen`, `?v=dokumente-ohne-zustand`, `?v=umsaetze-ohne-detail`, `?v=dokumente-gross`, `?v=dokumente-klein`.

## Done when

- [ ] The line between two columns can be dragged in every list with detail, by mouse and, with the focus on it, by the arrow keys. A double click gives the width of the design back. No column gets narrower than its least width, and the page of a document never narrower than 320 px: below that the values fold under the page.
- [ ] The sidebar folds to a rail of 56 px by its button and by Strg+B (⌘B): icons with their names as tooltips, the count of "Prüfen" and the intake stay in sight.
- [ ] The values of a document and the detail of a booking fold away by a button of the top bar that stays pressed while the column shows.
- [ ] In Dokumente the filter of the states ends, below a line, with "Zustand ausblenden". Chosen, the rows show no state and give its room to the name, the filter reads "Ausgeblendet", and the line of counts still says how many wait. Any other entry of the filter shows the states again. The filter lists each state with its count.
- [ ] The size of the whole view is Klein, Mittel or Groß (90, 100, 115 %), chosen in Einstellungen › Darstellung and by Strg with +, − and 0, as the zoom of the window. To be verified here at v2.tauri.app: how a webview is zoomed in Tauri 2 on WebView2 and WKWebView, and which permission it needs. The permission gets its row in [`knowledge/decision-app-shell.md`](../knowledge/decision-app-shell.md).
- [ ] Widths, the rail, the folded columns and the size are kept over a restart, per view where they belong to one. Where they are kept (not in the archive, they belong to the computer) is decided here and recorded.
- [ ] A view folds by the width it really has: with the size Groß a window of 1440 px behaves as one of 1252 px.
- [ ] The check of #51 opens every view with the rail, with a column folded and at Klein and Groß. Tests cover the least widths and what is kept.
- [ ] Seen by the owner on Windows.

## Out of scope

- Folding sections of an overview: rejected in the decision.
- Reading a document large and the focus of "Prüfen": #54.
