# Decision: checks decide what is filed by itself

**Decided:** 2026-11-25 with the owner (documents that pass every check are filed by themselves, statements without waiting, at most 10 % of documents wait) · applies to `src-tauri/src/checks.rs`, the stage `checked`, `documents.state`, `decisions`

## Decision

- The stage `checked` runs the checks of a document (`checks::decide`) in one transaction with the stage. A document that passes every check is `filed` and gets a row of `decisions` (migration 0009): the rule, the values it saw with their reader, what it changed and how to undo it. Every other one is `waiting`, with each failed check as its reason. The job ends in `filed` or `waiting` accordingly.
- Each check has a stable id and is stored in `field_checks` with its field and a German sentence, passed or not, so "Warum?" shows the passed ones and the waiting list the failed ones:
  - `check:kind`: the kind is an invoice, receipt, payslip or statement.
  - `check:present`, `check:source`, `check:agree`, per decisive value (the gross of an invoice or receipt, the Brutto and the Netto of a payslip): it was found, it stands in the text or the XML (a value read only from the page image does not), and two independent readers give the same value while none gives another. Readers count by family: the owner, the e-invoice, code, the model, the image. A value the owner confirmed passes alone.
  - `check:sum`: net plus VAT is the gross to the cent and the VAT fits the rate within one cent (an invoice without net and VAT passes, one with only one of them does not). Brutto minus the seven deductions is the Netto to the cent.
  - `check:duplicate`: no other document of the kind has the same sender, number (or date) and gross, or the same employer and month. Both of two such documents wait, the owner decides.
  - `check:undone`: the owner undid the automatic filing.
- The value that counts for a field is the one of the strongest reader: the owner, the e-invoice, code, the model, the image.
- A statement is filed by the rule `file.statement:1` without these checks, the owner's choice of 2026-11-25. Its bookings are read in stage 3, which brings their own checks. Everything else is filed by `file.checks-passed:1`.
- So that two independent readers exist, the model is asked for the decisive values even when code found them (#14 asked only for the missing ones).
- "Rückgängig" (`checks::undo`) marks the decision undone and lets the document wait. It is never filed by itself again, the check `check:undone` fails until the owner decides in #17.
- Documents of an older archive that were never checked (`state` `open`) get a job at the stage `read` at the next start (`checks::requeue_unchecked`), so the model reads them and the checks decide.

## Why

- In 1.0 an automatic acceptance stood on the model's own certainty and on one reading. Here a document is filed only when its decisive values stand in the document, add up, are read the same by two readers of different kinds, and it is no duplicate. Every decision can be explained and undone.

## Measured on 2026-11-26

`npm run measure -- --reader checks --archive <copy>` on a copy of the owner's archive (database and files, counts only), with `gemma4:12b-it-qat`, 69 seconds, the model unloaded afterwards:

- 122 of 132 documents are filed by themselves: the 110 statements, 11 of 18 invoices and the 1 receipt. 10 wait, 7.6 %, within the owner's limit of 10 %.
- Why they wait (a document can fail more than one check): code and model read a different gross 4, the kind is unclear 3 (the 2 documents of kind `other` and the scan, which the image named `other`), another document with the same sender, number and gross 2, the gross read only from the page image 2, net plus VAT is not the gross 2.

## "Prüfen": the review as questions (#17)

- The app has a sidebar (`SidebarRow` of the design system) with "Dokumente" and "Prüfen", which counts what waits. The path in the top bar reads "Prüfen › <group> › <document>", each earlier segment leads back.
- A waiting document belongs to the group of its first failed check, in the order undone, kind, present, source, agree, sum, duplicate (`review::groups`). A group lists its documents oldest first with their question.
- A question names the value, who read it and the sender, for example "Ist 49,99 € der Bruttobetrag von „Beispiel Stadtwerke GmbH“?". The page lies beside it with the line of the first reading marked.
- "Ja" stores the value as a value of the owner (reader `owner`, with the page and line of the reading). "Nein" shows the different readings beside each other and a field for the right value. A typed amount goes through `Cents::parse_de`, a refused one shows its reason. An unclear kind is answered by picking the kind. A failed sum, a possible duplicate and an undone filing are answered with "Ja" (accepted) or "Nein" (it keeps waiting).
- "Nein" (decided by the owner on 2026-11-26, after trying 0.2.0: "Bei Nein wird das Dokument nicht entfernt"):
  - Duplicate: "Nein, Doppel löschen …" asks once, then deletes the document (`documents::delete`). Every row about it goes in one transaction. The original moves to `Papierkorb` beside the archive folder, named by the start of its hash and its original name, until the owner empties it. The documents that waited as its duplicate are checked again and are filed when nothing else holds them.
  - After "Rückgängig": "Nein, zurückhalten" sets the document aside (a value of the owner `hold`). It stays unfiled, leaves "Prüfen", and the list shows it as "zurückgehalten".
  - Value question: "Nein" asks "Welcher Wert stimmt?", with the readings as buttons and a field. The document moves on once one is chosen, "Zurück zur Frage" returns.
- What the owner accepts is a value of the owner under `accept:<check>`. It turns that failed check into a passed one, "Von dir bestätigt: …". A document filed after such an answer gets the rule `file.owner-answered:1`. A value of the owner counts alone and stands above every reading, the readings beside it no longer disagree.
- After every answer the checks run again (`checks::decide`): the document is filed or waits with what is left, and "Prüfen" moves on to the next document of the group.
- Keys: J and K move in a list, Enter opens, 1 and 2 pick a reading once "Nein" opened them, Escape goes back (`src/review/keys.ts`, tested in `keys.test.ts`).
