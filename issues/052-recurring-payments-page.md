# #52 The page "Wiederkehrend" under Konten

**Goal:** The owner sees every recurring payment with what it costs in the month and what changed.
**Requirements**: FA-49 · **Depends on**: #34, #48

The rules: [`knowledge/decision-views.md`](../knowledge/decision-views.md). The picture: `?v=wiederkehrend`. What a recurring payment is: #34.

## Done when

- [x] "Konten › Wiederkehrend": the sum in the month and in the year, then every payment with its rhythm, its next day, its amount and "läuft", "gestiegen", "doppelt?" or "beendet". "gestiegen" joins the vocabulary of rule 11 with this change.
- [x] Beside the list the chosen payment: its amounts over the last twelve months, the next expected day, the sum in the year, its category, "Buchungen zeigen" as a jump to Umsätze, and "Ist keine wiederkehrende Zahlung", which stays (#34).
- [x] Konten leads to the page with the count and the sum in the month.
- [ ] Tests cover what the page derives for a row. Seen by the owner on the real archive.

## Out of scope

- The section in Auswertungen: #38. Savings hints from it: #39.

**Status:** built on 2026-11-27, waiting for the owner's look on the real archive.

- `list_recurring` now gives each payment its category, its bookings and its last twelve payments. `src/money/recurrence.ts` derives a row's state ("beendet", "doppelt?" when a savings hint of two running payments of one kind names it, "gestiegen" when its amount rose, else "läuft") and the sums in the month and the year of the running payments out; tested.
- "gestiegen" joins the words of rule 11 with this change, and `npm run check:views` knows it.
- The last twelve payments stand as gray bars on one scale, the last in the accent (rule 12).
- Seen in Chromium with the stand-in of the core; `npm run check:views`: 0 findings over 102 pages.
