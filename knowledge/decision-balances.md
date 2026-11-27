# Decision: balances over time in one pass, the bank's where it printed one

**Decided:** 2026-11-27 (#36, FA-40) · applies to `src-tauri/src/balances.rs`, `src-tauri/src/commands/balances.rs`

## Decision

- **Every day of an account** from its opening to the last day it knows (its last booking or the end of its last statement), the balance at the end of the day. The bookings are those `accounts::balance_on` counts, read once in day order and added up as the days pass: one query per account, not one per day.
- **The bank's balance is shown** on the day a statement closes, the one counted is kept beside it, and a difference is the bank's minus the counted, named as in "Konten" (#22).
- **The sum of all accounts** for every day from the first opening to the last day any account knows. An account counts from its opening, with its last balance after its last day (an account whose statements stopped did not empty itself). An account without an opening is left out.
- **To the window by month** (`balances_by_month`): each account and the sum at the end of each month, the last month at the last day known. Cents as `i32` and as German text. The daily series stays in the core for what needs it.

## Measured on 2026-11-27

`balances::tests` with invented statements: every day equal to `balance_on`, a booking dated with the day of the statement before included, the bank's balance shown and a difference kept, the sum of two accounts with different openings, an account without an opening, the ends of months, ten years of days in one pass (well under two seconds in a debug build).

A copy of the owner's archive (`examples/measure.rs --reader balances`, counts in `measurements/private/`):

| What | Count |
|---|---|
| Accounts with an opening | 3 |
| Days of all accounts | 5,732 |
| Days of the sum | 4,148 |
| Statement ends | 110 |
| Counted equal to the bank's | 110 |
| Time for all of it (release) | under 0.01 s |
