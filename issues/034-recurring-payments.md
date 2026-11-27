# #34 Recurring payments

**Goal:** Subscriptions, contracts, insurances and rent are known, with the next payment and a change of their amount.
**Requirements**: FA-38 · **Depends on**: #33

## Done when

- [x] A recurring payment is found from bookings of one party in a rhythm (monthly, quarterly, yearly) with the same or a near amount, at least three times. Its rhythm, its last and its next expected day and its amount in the year.
- [x] A changed amount is named with the day it changed ("seit März 2 € mehr"). One that stopped (twice its rhythm without a payment) is marked as ended, never removed.
- [x] The owner can say that a found one is none, and that stays.
- [ ] Tests with invented bookings. Measured on a copy of the owner's archive, counts only: how many were found per rhythm, and a sample checked by the owner.

**Status:** done on 2026-11-27 but for the sample the owner checks (`knowledge/decision-recurring.md`, "Open"). Tests and the counts on the copy are in.
