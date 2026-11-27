# #61 Correcting a value and the kind at the document

**Goal:** The owner corrects a wrong value or a wrong kind where it is seen, in Dokumente, and reaches the question of a waiting document from there.
**Requirements**: FA-61 · **Depends on**: #46, #47

As of 0.7.0 a value is corrected only by answering a question in "Prüfen". A document filed with a wrong value, or with the wrong kind, cannot be put right in Dokumente: its values are read-only (`Documents.tsx`), and the core has no command for it beside `answer_question`. A waiting document shows why it waits but no way to its question. "Neu lesen lassen" stays rejected (`knowledge/decision-views.md`, "Known limits").

## Done when

- [ ] Each value of a document in Dokumente has "Ändern": the owner types the right value, it is parsed by the core as in "Prüfen" (an amount as printed in German, a date, a text), refused with its reason at the field when it does not parse.
- [ ] The typed value is stored as a value of the owner (FA-15), the document goes through the checks again, and its name follows the template unless the owner named it.
- [ ] A value the document has none of can be added from the fields of its kind.
- [ ] "Art ändern …" in the menu of a document sets the kind. The values are read again for that kind, and the document is checked again. A statement with bookings is not changed this way: the dialog says so and names "Löschen …".
- [ ] A correction is a decision with "Rückgängig", which brings the read value back.
- [ ] A waiting document shows "Zur Frage" beside its reasons, which opens its first question in "Prüfen" with the arrow back to the document.
- [ ] A document already linked to a booking whose amount is corrected is held against that booking again: the link stays when it still fits, else it becomes a question.
- [ ] Tests in the core cover each of the points above with invented documents.

## Out of scope

- Reading a document again by its readers: rejected in `knowledge/decision-views.md`.
- Changing a booking of a statement: a statement is the bank's word.
