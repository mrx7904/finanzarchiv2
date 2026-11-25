# Decision: how a document is read, fixed readers first

**Decided:** 2026-11-25, with stage 2 · applies to `src-tauri/src/kinds.rs`, the readers of #11 to #15, the stages `classified` and `read` of the intake

## The kind by code (#10)

- The stage `classified` gives each document one of `invoice`, `payslip`, `receipt`, `statement`, `other` or `unknown`, from the signals printed in the text the stage `text` stored. No model: the kind decides which readers run, so it must come out the same every time and name why.
- The signals of each kind are one list in `kinds.rs`, lower case, matched only at the start of a word, so "Verdienstabrechnung" does not count as "Rechnung" and "Rechnungsnummer" counts once. The general word "rechnung" is no signal, it stands in reminders, letters and statements too.
- A payment confirmation ("Zahlungsbestätigung", "Quittung", "Kassenbon", "Amount paid", "Date paid" and others) makes a receipt before any count, because a receipt carries the signals of its invoice too (lesson of 1.0, the Stripe receipts beside their invoices).
- Otherwise the kind with the most signals wins when it has at least two. Fewer than two is `other`, a tie between the strongest two is `unknown`. No text at all (a scan, an image) is `unknown` until #15 reads the page as an image.
- The signals the kind rests on are stored as a JSON array in `documents.kind_signals` (migration 0005). A document read by an older version (text, but no signals) gets its kind at the next start (`documents::classify_missing`), once.

### Measured on 2026-11-25

- Generated corpus, `npm run measure -- --reader kind`: 25 of 25 invoices, 12 of 12 payslips, 6 of 6 statements, and the 5 scans `unknown` as they should be (`measurements/synthetic-kind.json`).
- The owner's 132 documents (a copy of the archive, `--archive`, counts only): 110 statements, 18 invoices, 1 receipt, 2 other, 1 unknown (the scan). All 110 statements carry "Kontoauszug" in their file name and "Alter Kontostand" and "Neuer Kontostand" in their text. Of the 22 others, every document whose name says "Rechnung" is an invoice. One file named as a "Beleg" is `other`, to be looked at with the readers of #11.
