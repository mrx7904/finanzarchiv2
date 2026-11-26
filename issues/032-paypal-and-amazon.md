# #32 PayPal and Amazon

**Goal:** Payments over PayPal count even without the seller's invoice, and Amazon orders find their invoice.
**Requirements**: FA-36 · **Depends on**: #31, #24

## Done when

- [ ] A mail counts as a PayPal payment mail only when it comes from PayPal's own domain. Code reads it: payee, amount, day and transaction code. The kind is receipt, and it is linked to its booking by #24.
- [ ] The seller's invoice first: when one comes (same amount, within 10 days, the seller's name or the order number), the PayPal mail becomes its receipt. Two mails of one payment (the same transaction code) are one.
- [ ] Amazon: the owner downloads the invoice into the inbox or the downloads (decided 2026-11-26, the app never signs in at Amazon). An order mail is matched to that invoice by its order number, and the invoice to its booking.
- [ ] Measured on the owner's mail, counts only.
