# #24 Documents linked to their bookings

**Goal:** Each invoice, receipt and payslip finds the booking that paid it, and each link can be explained and undone.
**Requirements**: FA-29 · **Depends on**: #20, #16

## Done when

- [x] Rules with stable ids, the surest first, their parameters in one module:
  - the invoice number in the purpose, with the amount;
  - the amount with the party, within the days around the due date;
  - a payslip's Netto with the employer, in the month.
- [x] A link is a row of `links` and a decision (rule, values seen, undo). One booking may pay several documents, the sums checked to the cent. Paid in parts is not built yet: no case in the owner's archive.
- [x] A link no rule is sure of (two bookings fit, or the name differs) is a question in "Prüfen": "Hat diese Buchung die Rechnung von … bezahlt?", with both side by side.
- [x] Measured on the owner's copy, counts only: documents linked, by which rule, questions left.
