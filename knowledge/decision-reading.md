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

## Values with their source, the code reader of invoices (#11)

- A value is a row of `field_values` (migration 0006): document, field, the value as printed, the normalized value (whole cents, an ISO date, a percent, text), the reader, the page and the box of its line, the time. The reader is never empty, the table refuses it. Checks get `field_checks`. The values of the owner are rows with the reader `owner` and are never replaced by a reader.
- Readers implement one trait (`readers::Reader`, `name` and `read`) and get the kind and the lines of every page with their boxes. `readers::for_kind` names the readers of a kind. The stage `read` of the intake runs them and replaces only their own earlier values, in one transaction with the stage.
- `READERS_VERSION` (now 1) is stored per document (`documents.read_version`). At start the app reads again every document an older version read (`readers::read_outdated`), so a better reader reaches the documents already filed.
- The code reader of invoices (`readers::invoice`) reads number, invoice date, due date, net, VAT rate and amount, gross, IBAN and sender from labelled lines. Labels are lower case and count only at the start of a word. The amount of a labelled line is its last one, a date its first one, an IBAN only with a valid check sum. The weak labels of the gross ("gesamt", "summe", "total") skip a line that names the net or the tax. The sender is the first line of page 1 that is neither a label, a date nor an amount. Amounts go through `Cents::parse_de`, dates through code, never a model.

### Measured on 2026-11-25

- Generated corpus, `npm run measure -- --reader code`: every field of the 25 invoices with text right, 25 of 25 each (`measurements/synthetic-code.json`). Payslips and statements are 0 until #13 and stage 3.
- The owner's 18 invoices, counts of fields found (not checked against a truth, the owner has confirmed none yet): sender 18, gross 15, invoice number 11, VAT 10, invoice date 8, IBAN 6, net 4, VAT rate 2, due date 1. The labels were widened after a count of the words beside dates and amounts in these invoices: "Rechnung vom" before most dates, "Zahlbetrag", "Gesamtpreis", "Gesamtsumme" and "Offener Betrag" beside the gross, and "Geburtsdatum" on 9 dental invoices, which is why "Datum" counts only at the start of a word. What code does not find is read by the model in #14.
