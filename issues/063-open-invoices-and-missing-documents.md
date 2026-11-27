# #63 Invoices open and paid, bookings without a document

**Goal:** The owner sees which invoices are not paid yet and which spendings have no document, without looking at each row.
**Requirements**: FA-63 · **Depends on**: #24, #46, #48

The archive knows the due date of an invoice (FA-16) and the booking that paid it (FA-29), but as of 0.7.0 no list tells "offen" from "bezahlt": the booking shows only inside the values of one document. Nor does "Umsätze" tell which bookings have no document. 1.0 had both ("Rechnungen … bezahlt und offen", the lists "Ohne Beleg" and "Beleg ohne Buchung").

## Done when

- [ ] A row of an invoice in Dokumente says "bezahlt" with the day, "offen" with the due date, or "überfällig" when the due date has passed and no booking paid it. The core decides, with the rules recorded in `knowledge/decision-payments.md`: what counts as paid (a link, a paired PayPal receipt), and from when an invoice without a booking is no longer expected (older than the first statement of every account).
- [ ] The filter "Zustand" of Dokumente gains "offen" and "überfällig", each with its count.
- [ ] "Umsätze" gains the filter "Beleg": "mit Beleg", "ohne Beleg". Moves between own accounts, salary and bookings of a party marked as needing none are not counted as "ohne Beleg". "Braucht keinen Beleg" at a booking and at a party sets and undoes that.
- [ ] "Start" lists "n Rechnungen überfällig" as a step that opens the filtered list.
- [ ] The chat gains a lookup for open invoices, checked like its other numbers (`knowledge/decision-chat.md`).
- [ ] Tests in the core with invented invoices and bookings: paid by a link, paid by a receipt, open, overdue, too old to expect.
- [ ] Measured on a copy of the owner's archive, counts only: how many invoices are open and overdue, and a sample of 10 checked by the owner.

## Out of scope

- Reminders and notifications: `spec/concept.md`, "Not in scope".
- Paying an invoice from the app.
