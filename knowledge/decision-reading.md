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
- Readers implement one trait (`readers::Reader`, `name` and `read`) and get the kind, the lines of every page with their boxes and the XML of an e-invoice. A reader is named with the version of the readers (`code:3`); reading again replaces the values of every version of the same reader. `readers::for_kind` names the readers of a kind. The stage `read` of the intake runs them and replaces only their own earlier values, in one transaction with the stage.
- `READERS_VERSION` (now 3) is stored per document (`documents.read_version`). At start the app reads again every document an older version read (`readers::read_outdated`), so a better reader reaches the documents already filed.
- The code reader of invoices (`readers::invoice`) reads number, invoice date, due date, net, VAT rate and amount, gross, IBAN and sender from labelled lines. Labels are lower case and count only at the start of a word. The amount of a labelled line is its last one, a date its first one, an IBAN only with a valid check sum. The weak labels of the gross ("gesamt", "summe", "total") skip a line that names the net or the tax. The sender is the first line of page 1 that is neither a label, a date nor an amount. Amounts go through `Cents::parse_de`, dates through code, never a model.

### Measured on 2026-11-25

- Generated corpus, `npm run measure -- --reader code`: every field of the 25 invoices with text right, 25 of 25 each (`measurements/synthetic-code.json`). Statements are 0 until stage 3, payslips see #13.
- The owner's 18 invoices, counts of fields found (not checked against a truth, the owner has confirmed none yet): sender 18, gross 15, invoice number 11, VAT 10, invoice date 8, IBAN 6, net 4, VAT rate 2, due date 1. The labels were widened after a count of the words beside dates and amounts in these invoices: "Rechnung vom" before most dates, "Zahlbetrag", "Gesamtpreis", "Gesamtsumme" and "Offener Betrag" beside the gross, and "Geburtsdatum" on 9 dental invoices, which is why "Datum" counts only at the start of a word. What code does not find is read by the model in #14.

## E-invoices from their XML (#12)

- A ZUGFeRD 2, Factur-X or XRechnung invoice in a PDF carries its data as an XML attachment. The stage `text` asks PDFium for the attachments of every PDF, takes `factur-x.xml`, `zugferd-invoice.xml` or `xrechnung.xml` first and else any `.xml`, and stores it in `e_invoices` (migration 0007, at most 5 MB). `documents.attachments_read` marks that a PDF was looked at, so the PDFs of an older archive are looked at once at the next start (`e_invoice::extract_missing`), before `read_outdated`.
- The reader `e-invoice` (`readers::e_invoice`) parses it with `quick-xml` and reads number, invoice date, due date, seller, net, VAT, VAT rate, gross and IBAN from their elements in the Cross Industry Invoice, by the full path below the root, so the name of the buyer or of a product is never the sender. The source of each value is that path, stored in `field_values.element` beside the empty line. Amounts go through `Cents::parse_decimal` (a point, no grouping, decimals past the cent must be zero, nothing is rounded), dates only in the form `102` (`YYYYMMDD`), a rate only when it is the one rate and whole, an IBAN only with a valid check sum. Amounts in another currency than euro are left out and said.
- The values of the XML are what the sender's system wrote, so they count as exact. The code reader still reads beside it: where both agree the printed page and the data match, where they differ the checks of #16 decide.
- The root must be `CrossIndustryInvoice` in the namespace `urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100`. Checked on 2026-11-25 in the reference messages of the KoSIT test suite for XRechnung 3.0 (`itplr-kosit/xrechnung-testsuite`, last commit 2026-09-04). The current releases are ZUGFeRD 2.5.2 and Factur-X 1.09.2 of 2026-08-04 (FeRD, FNFE-MPE, CII syntax D22B, same root namespace) and XRechnung 3.0, in force since 2024-02-01 with 4.0 announced for spring 2027 (xeinkauf.de). ZUGFeRD 1 (another namespace) and the UBL syntax of XRechnung are not read: their XML gives the problem "keine Cross Industry Invoice" and the other readers read the page.
- Whether the XML could be read is a check in `field_checks` (`e-invoice:readable`, for the whole document), with the German reason when it could not (broken, not UTF-8, not CII, too large, foreign currency). A broken XML gives no values, and the code reader reads the page as for any invoice. An attached e-invoice is read whatever kind the document got.

### A standalone XRechnung XML is not taken in

**Decided:** 2026-11-25. An XRechnung sent as a bare `.xml` (common towards public authorities, rare towards a person) is refused by the intake like any other file type for now. The archive keeps documents a person can look at, and a bare XML has no page to show, the owner would see a document without a preview. When one arrives, the step is a reader for the UBL and CII syntax and a rendered view of the XML. Until then such a file is refused with its type, as today.

### Measured on 2026-11-25

- Generated corpus, `npm run measure -- --reader e-invoice`: the e-invoice of the corpus (`rechnung-30`, its XML now with due date and IBAN) has every field right, 9 of 9 (`measurements/synthetic-e-invoice.json`).
- The owner's 132 PDFs (a copy of the files, `--files`, counts only): none carries an XML. The reader waits for the first e-invoice, the code reader keeps reading the owner's invoices.

## Payslips by code (#13)

- The code reader of payslips (`readers::payslip`, named `code` like the one of invoices, one of them runs per kind) reads the employer, the month, the Brutto, Lohnsteuer, Solidaritätszuschlag, Kirchensteuer, the four insurances and the Netto from labelled lines, the stronger label first ("Gesamtbrutto" before "Brutto", "Auszahlungsbetrag" before "Netto").
- A deduction is the amount deducted, never its rate (lesson of 1.0). For every reader a figure followed by "%" is a rate and no amount (`parse::amounts`), so "Krankenversicherung 7,30 % 348,00" and "Rentenversicherung 372,00 9,30 %" both give the amount. A line of a deduction that also names the employer's share ("Arbeitgeber", "AG-Anteil"), a Brutto as its base ("KV-Brutto") or a total of the year ("Jahres…", "kumuliert") is skipped.
- The month comes from the first line that names the period ("Verdienstabrechnung", "Entgeltabrechnung", "Abrechnungsmonat", "Zeitraum" and others) and a month: "März 2026", "03/2026", "11.2026" or the month of its first date. It is stored as `YYYY-MM`, the printed form stays beside it. The employer is the first line of page 1 that is neither the title, a date nor an amount.
- `READERS_VERSION` went to 3 with this reader, so every document is read again once at the next start.

### Measured on 2026-11-25

- Generated corpus, `npm run measure -- --reader code`: every field of the 12 payslips right, 12 of 12 each, and the 25 invoices still 25 of 25 each (`measurements/synthetic-code.json`).
- The owner's archive (a copy of the database, `--archive`, counts only): no payslip among the 132 documents, so there is nothing to count yet. The counts of the 18 invoices are unchanged by the rule on rates (sender 18, gross 15, number 11, VAT 10, date 8, IBAN 6, net 4, rate 2, due date 1).
