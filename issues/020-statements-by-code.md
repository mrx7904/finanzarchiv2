# #20 Statements read by code, each proved by its balance

**Goal:** Every PDF statement of the owner's banks is read by code into its account and bookings, and a statement whose bookings do not add up to its new balance waits.
**Requirements**: FA-25 · **Depends on**: #16

## Done when

- [ ] Readers by code for the layouts of the owner's banks, named by their layout and never by the bank (`decision-public-repository.md`). The layouts are found from the owner's statements read masked on this computer (every digit a 9, every word outside the bank's vocabulary a `w`), and nothing of their content enters the repository. The corpus gets generated statements in each layout with invented values.
- [ ] A statement gives bank, IBAN, number, period, old and new balance. It also gives its bookings: day, day of value, amount in cents with its sign, other side, purpose and kind. They go into `bookings`, with the statement and the line of each as their source. The year of a booking in December comes from the period.
- [ ] The check `check:balance` (old balance plus the bookings is the new balance to the cent) replaces the rule `file.statement:1`. A statement that adds up is filed, one that does not waits with the difference. The first statement of an IBAN makes its account.
- [ ] Measured on a copy of the owner's archive, counts only: statements read, statements adding up, bookings.

## Out of scope

- CSV, belongs to #21. The balance per day, belongs to #22. Moves, belong to #23.
