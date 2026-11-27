# #59 Failures show, and an answer that is overtaken does not count

**Goal:** When the core refuses or fails, the owner reads it at the place of the action, and a view never shows the answer to a question it no longer asks.
**Requirements**: FA-59 · **Depends on**: #46, #48

Found by reading the code on 2026-11-27. Many calls drop an error without a word, some views show an older answer over a newer one, and one rule lives in the window.

## Done when

- [ ] Every call of a command in `src/` handles `status: "error"`. A check in `scripts/check.mjs` fails on a call whose error branch is missing, with its probe in `CODE_STEPS`.
- [ ] These failures show their message where the owner acted: deleting a document, loading the values of a document, loading and paging "Umsätze", choosing a category, linking and unlinking a document, undoing a move, deleting a chat, "Rückgängig" of a filing.
- [ ] "Start" tells a failed lookup from a zero: a step whose source failed reads "ließ sich nicht prüfen", and the sentence on top never says that nothing waits while a lookup failed.
- [ ] In "Umsätze" the search waits 200 ms after the last key, and an answer to an older search, year or category is thrown away. The same holds for the questions of "Prüfen" and the values of a document. A test answers two requests in the wrong order and expects the newer one on the screen.
- [ ] Choosing a category in "Umsätze" keeps the rows already loaded and the place scrolled to.
- [ ] The "×" of the mark of a jumped text clears the text only, the "×" of the mark of a period or category clears that only.
- [ ] Typing in the search of "Umsätze" while a jumped text stands replaces the jumped text.
- [ ] Which account misses a statement comes from the core as a field of the account, not from the word "fehlt" in a sentence (`Start.tsx`, the rule "The UI holds no business rule" of `knowledge/conventions.md`).
- [ ] "Bestellungen" reloads when the intake changed, as the other lists do.
- [ ] The focus of "Prüfen" ends when the owner leaves "Prüfen", and the keys of "Prüfen" do nothing while a dialog is open.

## Out of scope

- A log of what failed: #70.
