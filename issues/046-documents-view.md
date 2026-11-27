# #46 The view "Dokumente" as designed

**Goal:** The owner finds every document in one list, sees its page and its read values beside it, and reaches the booking that paid it.
**Requirements**: FA-49 · **Depends on**: #44, #45

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The pictures: `?v=dokumente`, `?v=dokumente-leer`, `?v=bestellungen`, `?v=dialog-dokument-loeschen`, `?v=dialog-umbenennen`, `?v=dialog-rechnung-zuordnen`, and Dokumente at 960 × 600.

## Done when

- [x] Three columns above 1300 px: the list with its search, the kind and the state as filters and a line with the counts, the page of the chosen document, and its values with where each stands, the booking that paid it, and "Warum?" and "Rückgängig". Below 1300 px the values stand under the page.
- [x] The top bar holds "Importieren" as a menu and "…". The six buttons above the list are gone: Ordner, Postfächer and Namensvorlagen are reached in Einstellungen (#49).
- [x] The menu at the document: "Umbenennen …", "Datei im Ordner zeigen", "Löschen …". Deleting asks with what goes and what stays. Whether "Neu lesen lassen" is offered is decided here with what the core can do.
- [x] "Bestellungen" is a page below Dokumente: every order with "fehlt" or "vollständig", beside it its positions, its payment and its invoice, with "Rechnung zuordnen …" as a dialog that lists the invoices without an order, the likeliest first. Whether an order can be marked as needing no invoice is decided here.
- [x] Empty, loading and an error look as rules 14 to 16 say. The list stays quick with the owner's archive: measured with the number of documents and the time until the list stands.
- [x] Every state word is one of rule 11. Tests cover what the view derives for a row (state, detail line, the line of counts).
- [ ] Seen by the owner on the real archive, on Windows.

## Out of scope

- The pages of Einstellungen the buttons moved to: #49.
- Exporting as a table: not in the core, a requirement of its own when the owner asks for it.

**Status:** built on 2026-11-27, waiting for the owner's look on the real archive.

- Decided by Fable 5.2 on the owner's behalf (`knowledge/decision-views.md`, "Known limits"): no "Neu lesen lassen"; "Braucht keine Rechnung" at an order, undone by "Rückgängig" (migration 0027, `orders::set_needs_invoice`).
- New in the core: `document_detail` (the read values with where each stands: "gelesen", "stimmt" when a check of agreement or sum passed, "vom Absender" from an e-invoice, "von dir"; and the bookings that paid it), `delete_document` (a statement with its bookings, the twins of a duplicate judged again), `reveal_document`, `order_invoice_candidates` (the same total first, then the nearest day), `assign_order_invoice`, `set_order_needs_invoice`. Deleting a document or a statement now takes its tax assignments with it, which before #46 would have failed on the foreign key.
- An order shows its invoice, not its positions or its payment: the mails give the core neither yet.
- Measured on a copy of the owner's archive (`examples/measure.rs --reader documents-list`): 353 documents, the rows the window gets in 96 ms (release, best of five), below the 200 ms after which the gray rows show.
- Seen in Chromium with the stand-in of the core (`src/dev/mock.ts`, `?mock` in the dev server, invented data): three columns at 1440 × 900, the values under the page at 1200 × 800, "Bestellungen" with its path, a menu opened and chosen by keyboard. Found and fixed while looking: the intake's popover stood open, because a display class overrode the hiding of a closed popover; the page now fits its column.
