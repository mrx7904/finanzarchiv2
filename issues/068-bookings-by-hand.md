# #68 Accounts without statements, and the opening balance

**Goal:** Money that no statement tells, such as cash, has its account and its bookings, and an account can be given the balance it started with.
**Requirements**: FA-68 · **Depends on**: #48

`spec/concept.md` says: "Cash and accounts without statements get bookings too". As of 0.7.0 an account is made only by its first statement and a booking only by a statement. "Buchung von Hand" was put off on 2026-11-27 on the owner's behalf, because a booking by hand would double one once the statement comes, with the note that accounts without statements need their own kind (`knowledge/decision-views.md`, "Known limits"). An account whose first statement names no old balance reads "ohne Startsaldo" and nothing sets one.

**Needs the owner's word before the work begins:** whether cash and other accounts without statements belong into 2.0 now. Without it only the opening balance is built.

## Done when

- [ ] An account whose statements give no opening balance can be given one with its day, in the menu of the account. It is stored as a value of the owner and shows as such, and the balance against the bank (FA-27) counts from it.
- [ ] If the owner says yes: "Konto anlegen …" makes an account of the kind "ohne Kontoauszug" with a name and an opening balance. Only such an account takes bookings by hand: day, amount, direction, party, purpose, category. A statement can never be booked into it, so nothing is doubled.
- [ ] A booking by hand can be changed and deleted, each a decision with "Rückgängig". A booking of a statement cannot.
- [ ] A move between an account with statements and one without is made from the booking of the statement ("Umbuchung nach …"), so both balances change and neither income nor spending counts (FA-28).
- [ ] Bookings by hand count in the evaluations, the recurring payments and the taxes as every booking does, and show their source as "von dir".
- [ ] Tests in the core with invented accounts: the opening balance, a booking by hand, its change and undoing, a move into cash, the refusal of a statement for such an account.

## Out of scope

- Direct bank connections: `spec/concept.md`, "Not in scope".
- Daily budget planning.
