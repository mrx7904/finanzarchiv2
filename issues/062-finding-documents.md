# #62 Finding a document by what stands in it

**Goal:** The owner finds a document by a word of its text, its sender, its number or its amount, narrows by time and sorts the list.
**Requirements**: FA-62 · **Depends on**: #46

As of 0.7.0 the search of Dokumente and the quick search look at the name and the original file name only (`filterRows` in `src/documents/describe.ts`). The text of every page is in the archive since #06, and 1.0 had a full-text search (`docs/analyse-finanzarchiv-1.0.md`, "Archiv"). The list has no period and one fixed order.

## Done when

- [ ] The search of Dokumente finds by the text of the pages, the sender, the invoice number, the order number and the amount as the owner types it ("119", "119,00"). The core searches, the window only shows.
- [ ] To decide and record in `knowledge/decision-archive-file.md` before the work: FTS5 of the bundled SQLite or a plain query, measured on the owner's archive. To verify: FTS5 is compiled into `rusqlite` with the feature `bundled` (the documentation of `rusqlite` on the day of the work).
- [ ] A row found by its text shows the line that matched under its name, with the word marked. Opening it turns to the page of the match.
- [ ] A filter "Zeitraum" (year, month, from and to) by the date of the document, with the day it came as the fallback.
- [ ] The list sorts by "Neueste zuerst", "Datum des Dokuments", "Name" and "Betrag". The choice is kept as the widths are (#53).
- [ ] The filter "Art" lists every kind the archive holds, "Art offen" included.
- [ ] The quick search finds documents by sender and number too, at most 20 hits of the text search.
- [ ] Measured on a copy of the owner's archive: a search answers in under 100 ms.
- [ ] Tests in the core with invented documents: a hit by text, by number, by amount, no hit, umlauts and "ß".

## Out of scope

- Searching inside the open document: a requirement of its own, as in #54.
- Saved views, as 1.0 had: decided when the owner asks for it.
