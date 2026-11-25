# #10 The kind of a document by code

**Goal:** The stage `classified` of the intake gives every document its kind from the signals printed in its text, and the list shows it in German.
**Requirements**: FA-14 · **Depends on**: none

## Done when

- [x] Kinds are `invoice`, `payslip`, `receipt`, `statement`, `other` and `unknown`, the signals of each in one module with their reasons (lesson of 1.0: `kindBySignals`, payment confirmation first, two signals of a kind needed, "unknown" when two kinds tie).
- [x] The kind and the signals it rests on are stored for the document. A scan and an image stay `unknown` until stage 2 reads them as images (#15).
- [x] `npm run measure` gains the reader `kind` and reports per kind how many of the generated corpus it named right. Every invoice, payslip and statement with text is named right, shown in `measurements/`.
- [x] Measured once on the owner's 132 documents with counts only, the counts stand in the commit.
- [x] The list says "Rechnung", "Gehaltsabrechnung", "Zahlungsbeleg", "Kontoauszug", "Sonstiges" or "Art offen".

## Out of scope

- Reading the values, belongs to #11 and #13
- Reading a scan, belongs to #15
