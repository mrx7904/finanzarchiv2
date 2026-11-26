# #22 The balance of each account against the bank

**Goal:** The app knows the balance of every account on every day and shows where it differs from the bank or where a statement is missing.
**Requirements**: FA-27 · **Depends on**: #20

## Done when

- [x] `accounts` with bank, IBAN, kind and an opening balance. The opening balance is taken from the earliest statement and moved when an earlier one arrives, never over one the owner typed.
- [x] `balance_on(account, day)` in code, held against every balance a statement printed. A difference is a finding with its days and amounts. So is a gap between the new balance of one statement and the old balance of the next.
- [x] Property tests over the balance: bookings in any order give the same balance, and a statement in two parts gives the same as one.
- [x] Measured on the owner's copy: every account's balance equals the bank's on its last day, counts only.
