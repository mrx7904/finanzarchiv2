# #23 Moves between own accounts

**Goal:** Money moved between the owner's own accounts counts in both balances and as neither income nor spending.
**Requirements**: FA-28 · **Depends on**: #22

## Done when

- [x] A booking is a move (`booking_pairs`) when its other side is the IBAN of an own account, or when the bank marks it as a move to a linked account. With statements on both sides the two bookings are paired (same day, same amount, opposite direction). Otherwise the app makes the other half, marked as made.
- [x] The bank itself is never an own account, even when the purpose names an own IBAN (lesson of 1.0: dividends of a share of the bank counted as moves).
- [x] Each pairing is a decision with its rule, "Warum?" and "Rückgängig". The core is done here, the buttons come with the views of #26.
- [x] Measured on the owner's copy: pairs and made halves, counts only.
