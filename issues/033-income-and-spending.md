# #33 Income and spending from the bookings

**Goal:** The archive tells what came in and what went out, per month and per year, and why.
**Requirements**: FA-37 · **Depends on**: #23, #25

## Done when

- [x] Income and spending per month and per year from the bookings that are not replaced, the moves of #23 left out. A refund (money in from a party the owner paid) lowers the category of what it refunds, in the year it came back.
- [x] By category and by party, each sum with its count, "Ohne Kategorie" last. Every sum opens its bookings (the ids for the filter of "Umsätze", the click itself with #38).
- [x] The same figure for the year before, so a year compares with the last.
- [x] Tests with invented bookings: moves, refunds, a booking replaced by its CSV twin, a year boundary. Measured on a copy of the owner's archive, counts only: the sums of a year against the bookings one by one.

**Done:** 2026-11-26 ([decision-evaluations](../knowledge/decision-evaluations.md)). On the owner's archive every spending is "Ohne Kategorie" so far.
