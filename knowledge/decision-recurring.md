# Decision: recurring payments are chains of a party in a rhythm

**Decided:** 2026-11-27 (#34, FA-38) · applies to `src-tauri/src/recurring.rs`, `src-tauri/src/commands/recurring.rs`, the table `recurring_none` (migration 0023)

## Decision

- **From the bookings that count** (#33): none replaced by its CSV twin, no move between own accounts. Grouped by party and direction, so a salary is a recurring payment as much as rent.
- **A chain per rhythm.** From a payment the next one in the rhythm (monthly 25 to 36 days, quarterly 80 to 100, yearly 350 to 380), or one missed (twice the rhythm), with the amount nearest to the last: within a fifth, at least a euro, or a new amount that comes again in the rhythm (a changed price). Monthly first, then quarterly, then yearly, each booking in one chain at most. A chain needs three payments.
- **Shopping is no recurring payment.** A chain with more bookings of a near amount between its payments than half its own is dropped: the purchases at a shop form a chain by chance, a contract does not have its twins in between.
- **Two contracts of one party are two**: their amounts are not near, so they form two chains.
- **What it says:** its rhythm, the first day of the run, the last and the next day, the count, the last amount and that amount times the payments of a year. A changed amount is named when the last amount differs from one paid at least twice before ("seit März 2026 2,00 € mehr"); a bill that differs every month names none.
- **Ended, never removed.** Twice its rhythm without a payment ends it, judged by the last day the statements read reach (or the last booking), never by today alone: a statement comes weeks after its month. A pause longer than that starts a new run, the old one stays as ended.
- **"Ist keine" stays.** The owner's no is a row of `recurring_none`, named by party, direction, rhythm and the first day of the run, which survive a statement read again (the booking ids may not). No foreign key: a merged party leaves rows that name nothing.
- **To the window** as `RecurringRow` (`list_recurring`, `not_recurring`): cents as `i32` and as German text, days as `YYYY-MM-DD`. The page is #52, the section in Auswertungen #38.

## Measured on 2026-11-27

`recurring::tests` with invented bookings: monthly with its next day and its year, quarterly and yearly a few days late, a changed amount and a price that jumps by 70 % and stays, a bill that differs every month, a missed month with a one-off bill between, one that stopped, a salary, purchases every few days, two contracts of one party, a move between own accounts, ended by the last statement and not by today, the owner's no.

A copy of the owner's archive, counts only (`examples/measure.rs --reader recurring`, the counts and a sample with names in `measurements/private/`):

| What | Count |
|---|---|
| Found | 44 |
| monthly out / in | 29 / 4 |
| quarterly out / in | 6 / 1 |
| yearly out / in | 2 / 2 |
| Ended | 39 |
| With a changed amount | 8 |

The first version (groups of near amounts, three of four gaps in the rhythm) found 18. It lost every payment whose price jumped by more than a fifth and every long run with a few odd gaps; the chains found 44.

Most are ended because the main account of the archive has 38 to 47 bookings a month until July 2026 and 2 to 12 a month in the statements of August to October: the payments do not run over it any more. That is the archive, not the rule.

**Open:** the sample checked by the owner (`measurements/private/recurring-sample.json`), asked when the owner is back.
