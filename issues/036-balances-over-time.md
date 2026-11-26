# #36 Balances over time

**Goal:** The balance of each account and of all is known for every day the archive covers.
**Requirements**: FA-40 · **Depends on**: #22

## Done when

- [ ] The balance of an account at the end of each day from its opening and its bookings, the same as `accounts::balance_on`, and the sum of all accounts.
- [ ] Where the bank printed a balance, it is the one shown, and a difference to the bookings is named as in "Konten".
- [ ] Quick for ten years of daily balances (one pass over the bookings, not a query per day).
- [ ] Tests with invented statements. Measured on a copy of the owner's archive: the balance at each statement's end against the bank's.
