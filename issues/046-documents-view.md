# #46 The view "Dokumente" as designed

**Goal:** The owner finds every document in one list, sees its page and its read values beside it, and reaches the booking that paid it.
**Requirements**: FA-49 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=dokumente`, `?v=dokumente-leer`, `?v=bestellungen`, `?v=dialog-dokument-loeschen`, `?v=dialog-umbenennen`, `?v=dialog-rechnung-zuordnen`, and Dokumente at 960 × 600.

## Done when

- [ ] Three columns above 1200 px: the list with its search, the kind and the state as filters and a line with the counts, the page of the chosen document, and its values with where each stands, the booking that paid it, and "Warum?" and "Rückgängig". Below 1200 px the values stand under the page.
- [ ] The top bar holds "Importieren" as a menu and "…". The six buttons above the list are gone: Ordner, Postfächer and Namensvorlagen are reached in Einstellungen (#49).
- [ ] The menu at the document: "Umbenennen …", "Datei im Ordner zeigen", "Löschen …". Deleting asks with what goes and what stays. Whether "Neu lesen lassen" is offered is decided here with what the core can do.
- [ ] "Bestellungen" is a page below Dokumente: every order with "fehlt" or "vollständig", beside it its positions, its payment and its invoice, with "Rechnung zuordnen …" as a dialog that lists the invoices without an order, the likeliest first. Whether an order can be marked as needing no invoice is decided here.
- [ ] Empty, loading and an error look as rules 14 to 16 say. The list stays quick with the owner's archive: measured with the number of documents and the time until the list stands.
- [ ] Every state word is one of rule 11. Tests cover what the view derives for a row (state, detail line, the line of counts).
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- The pages of Einstellungen the buttons moved to: #49.
- Exporting as a table: not in the core, a requirement of its own when the owner asks for it.
