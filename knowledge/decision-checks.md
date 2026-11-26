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
